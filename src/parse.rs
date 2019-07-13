use bstr::ByteSlice;
use crate::{
    errors::{LoadJointsError, LoadMotionError},
    fraction_seconds_to_duration, Axis, Bvh, Channel, ChannelType, EnumeratedLines, JointData,
};
use lexical::try_parse;
use mint::Vector3;
use smallvec::SmallVec;
use std::{convert::TryFrom, mem, str};

impl Bvh {
    // @TODO: Remove panics
    /// Logic for parsing the data from a `BufRead`.
    pub(crate) fn read_joints(
        &mut self,
        lines: &mut EnumeratedLines<'_>,
    ) -> Result<(), LoadJointsError> {
        const HEIRARCHY_KEYWORD: &[u8] = b"HIERARCHY";

        const ROOT_KEYWORD: &[u8] = b"ROOT";
        const JOINT_KEYWORD: &[u8] = b"JOINT";
        const ENDSITE_KEYWORDS: &[&[u8]] = &[b"End", b"Site"];

        const OPEN_BRACE: &[u8] = b"{";
        const CLOSE_BRACE: &[u8] = b"}";

        const OFFSET_KEYWORD: &[u8] = b"OFFSET";
        const CHANNELS_KEYWORD: &[u8] = b"CHANNELS";

        #[derive(Debug, Eq, PartialEq)]
        enum ParseMode {
            NotStarted,
            InHeirarchy,
            Finished,
        }

        #[allow(unused)]
        #[derive(Eq, PartialEq)]
        enum NextExpectedLine {
            Hierarchy,
            Channels,
            Offset,
            OpeningBrace,
            ClosingBrace,
            JointName,
            RootName,
        }

        let mut joints = vec![];
        let mut curr_mode = ParseMode::NotStarted;
        let mut curr_channel = 0usize;
        let (mut curr_index, mut curr_depth) = (0usize, 0usize);
        let mut next_expected_line = NextExpectedLine::Hierarchy;

        let mut curr_joint = JointData::empty_root();
        let mut in_end_site = false;
        let mut pushed_end_site_joint = false;

        #[inline]
        fn get_parent_index(joints: &[JointData], for_depth: usize) -> usize {
            joints
                .iter()
                .rev()
                .find(|jd| jd.depth() == for_depth.saturating_sub(2))
                .and_then(|jd| jd.private_data().map(|p| p.self_index))
                .unwrap_or(0)
        }

        for (line_num, line) in lines {
            let line = line?;
            let line = line.trim();

            let mut tokens = line.fields_with(|c: char| c.is_ascii_whitespace() || c == ':');

            let first_token = match tokens.next() {
                Some(tok) => tok,
                None => continue,
            };

            match first_token.as_bytes() {
                HEIRARCHY_KEYWORD => {
                    if curr_mode != ParseMode::NotStarted {
                        panic!("Unexpected hierarchy");
                    }
                    curr_mode = ParseMode::InHeirarchy;
                    next_expected_line = NextExpectedLine::RootName;
                }
                ROOT_KEYWORD => {
                    if curr_mode != ParseMode::InHeirarchy
                        || next_expected_line != NextExpectedLine::RootName
                    {
                        panic!("Unexpected root: {:?}", curr_mode);
                    }

                    if let Some(name) = tokens.next() {
                        curr_joint.set_name(name);
                    } else {
                        panic!("Missing root name!");
                    }
                }
                OPEN_BRACE => {
                    curr_depth += 1;
                }
                CLOSE_BRACE => {
                    curr_depth -= 1;
                    if curr_depth == 0 {
                        // We have closed the brace of the root joint.
                        curr_mode = ParseMode::Finished;
                    }

                    if in_end_site {
                        if let JointData::Child {
                            ref mut private, ..
                        } = curr_joint
                        {
                            private.self_index = curr_index;
                            private.parent_index = get_parent_index(&joints, curr_depth);
                            private.depth = curr_depth - 1;
                        }

                        let new_joint = mem::replace(&mut curr_joint, JointData::empty_child());
                        joints.push(new_joint);
                        curr_index += 1;
                        in_end_site = false;
                        pushed_end_site_joint = true;
                    }
                }
                kw if kw == ENDSITE_KEYWORDS[0] => {
                    if tokens.next() == Some(ENDSITE_KEYWORDS[1]) {
                        in_end_site = true;
                    } else {
                        panic!("Unexpected end keyword");
                    }
                }
                JOINT_KEYWORD => {
                    if curr_mode != ParseMode::InHeirarchy {
                        panic!("Unexpected Joint");
                    }

                    if !pushed_end_site_joint {
                        if let JointData::Child {
                            ref mut private, ..
                        } = curr_joint
                        {
                            private.self_index = curr_index;
                            private.parent_index = get_parent_index(&joints, curr_depth);
                            private.depth = curr_depth - 1;
                        }

                        let new_joint = mem::replace(&mut curr_joint, JointData::empty_child());
                        joints.push(new_joint);

                        curr_index += 1;
                    } else {
                        pushed_end_site_joint = false;
                    }

                    if let Some(name) = tokens.next() {
                        curr_joint.set_name(name);
                    } else {
                        panic!("Missing joint name!");
                    }
                }
                OFFSET_KEYWORD => {
                    if curr_mode != ParseMode::InHeirarchy {
                        return Err(LoadJointsError::UnexpectedOffsetSection { line: line_num });
                    }

                    let mut offset = Vector3::from([0.0, 0.0, 0.0]);

                    macro_rules! parse_axis {
                        ($axis_field:ident, $axis_enum:ident) => {
                            if let Some(tok) = tokens.next() {
                                offset.$axis_field = try_parse(tok).map_err(|e| {
                                    LoadJointsError::ParseOffsetError {
                                        parse_float_error: e,
                                        axis: Axis::$axis_enum,
                                        line: line_num,
                                    }
                                })?;
                            } else {
                                return Err(LoadJointsError::MissingOffsetAxis {
                                    axis: Axis::$axis_enum,
                                    line: line_num,
                                });
                            }
                        };
                    }

                    parse_axis!(x, X);
                    parse_axis!(y, Y);
                    parse_axis!(z, Z);

                    curr_joint.set_offset(offset, in_end_site);
                }
                CHANNELS_KEYWORD => {
                    if curr_mode != ParseMode::InHeirarchy {
                        return Err(LoadJointsError::UnexpectedChannelsSection { line: line_num });
                    }

                    let num_channels: usize = tokens
                        .next()
                        .ok_or(LoadJointsError::ParseNumChannelsError {
                            error: None,
                            line: line_num,
                        })
                        .and_then(|tok| match try_parse(tok) {
                            Ok(c) => Ok(c),
                            Err(e) => Err(LoadJointsError::ParseNumChannelsError {
                                error: Some(e),
                                line: line_num,
                            }),
                        })?;

                    let mut channels: SmallVec<[Channel; 6]> = Default::default();
                    channels.reserve(num_channels);

                    while let Some(tok) = tokens.next() {
                        let channel_ty = ChannelType::try_from(tok).map_err(|e| {
                            LoadJointsError::ParseChannelError {
                                error: e,
                                line: line_num,
                            }
                        })?;
                        let channel = Channel::new(channel_ty, curr_channel);
                        curr_channel += 1;
                        channels.push(channel);
                    }

                    curr_joint.set_channels(channels);
                }
                _ => {}
            }

            if curr_mode == ParseMode::Finished {
                break;
            }
        }

        if curr_mode != ParseMode::Finished {
            return Err(LoadJointsError::MissingRoot);
        }

        self.joints = joints;
        self.num_channels = curr_channel;

        Ok(())
    }

    pub(crate) fn read_motion(
        &mut self,
        lines: &mut EnumeratedLines<'_>,
    ) -> Result<(), LoadMotionError> {
        const MOTION_KEYWORD: &[u8] = b"MOTION";
        const FRAMES_KEYWORD: &[u8] = b"Frames";
        const FRAME_TIME_KEYWORDS: &[&[u8]] = &[b"Frame", b"Time:"];

        macro_rules! last_line_num {
            () => {
                lines.last_enumerator().unwrap_or(0)
            };
        }

        lines
            .next_non_empty_line()
            .ok_or(LoadMotionError::MissingMotionSection {
                line: last_line_num!(),
            })
            .and_then(|(line_num, line)| {
                let line = line?;
                let line = line.trim();
                if line == MOTION_KEYWORD {
                    Ok(())
                } else {
                    Err(LoadMotionError::MissingMotionSection { line: line_num })
                }
            })?;

        self.num_frames = lines
            .next_non_empty_line()
            .ok_or(LoadMotionError::MissingNumFrames {
                parse_error: None,
                line: last_line_num!(),
            })
            .and_then(|(line_num, line)| {
                let line = line?;
                let line = line.trim();
                let mut tokens = line.fields_with(|c: char| c.is_ascii_whitespace() || c == ':');

                if tokens.next() != Some(FRAMES_KEYWORD) {
                    return Err(LoadMotionError::MissingNumFrames {
                        parse_error: None,
                        line: line_num,
                    });
                }

                let parse_num_frames = |token: Option<&[u8]>| {
                    if let Some(num_frames) = token.and_then(|b| str::from_utf8(b).ok()) {
                        try_parse::<usize, _>(num_frames)
                            .map_err(|e| LoadMotionError::MissingNumFrames {
                                parse_error: Some(e),
                                line: line_num,
                            })
                            .map_err(Into::into)
                    } else {
                        Err(LoadMotionError::MissingNumFrames {
                            parse_error: None,
                            line: line_num,
                        })
                    }
                };

                match tokens.next() {
                    Some(tok) if tok == b":" => parse_num_frames(tokens.next()),
                    Some(tok) => parse_num_frames(Some(tok)),
                    None => Err(LoadMotionError::MissingNumFrames {
                        parse_error: None,
                        line: line_num,
                    }),
                }
            })?;

        self.frame_time = lines
            .next_non_empty_line()
            .ok_or(LoadMotionError::MissingFrameTime {
                parse_error: None,
                line: last_line_num!(),
            })
            .and_then(|(line_num, line)| {
                let line = line?;
                let mut tokens = line.fields();

                let frame_time_kw = tokens.next();
                if frame_time_kw == FRAME_TIME_KEYWORDS.get(0).map(|b| *b) {
                    // do nothing
                } else {
                    return Err(LoadMotionError::MissingFrameTime {
                        parse_error: None,
                        line: line_num,
                    });
                }

                let frame_time_kw = tokens.next();
                if frame_time_kw == FRAME_TIME_KEYWORDS.get(1).map(|b| *b) {
                    // do nothing
                } else {
                    return Err(LoadMotionError::MissingFrameTime {
                        parse_error: None,
                        line: line_num,
                    });
                }

                let parse_frame_time = |token: Option<&[u8]>| {
                    if let Some(frame_time) = token {
                        let frame_time_secs = try_parse::<f64, _>(frame_time).map_err(|e| {
                            LoadMotionError::MissingFrameTime {
                                parse_error: Some(e),
                                line: line_num,
                            }
                        })?;
                        Ok(fraction_seconds_to_duration(frame_time_secs))
                    } else {
                        Err(LoadMotionError::MissingFrameTime {
                            parse_error: None,
                            line: line_num,
                        })
                    }
                };

                match tokens.next() {
                    Some(tok) if tok == b":" => parse_frame_time(tokens.next()),
                    Some(tok) => parse_frame_time(Some(tok)),
                    None => Err(LoadMotionError::MissingNumFrames {
                        parse_error: None,
                        line: line_num,
                    }),
                }
            })?;

        let expected_total_motion_values = self.num_channels * self.num_frames;

        self.motion_values.reserve(expected_total_motion_values);

        for (line_num, line) in lines {
            let line = line?;
            let tokens = line.fields();
            for (channel_index, token) in tokens.enumerate() {
                let motion = try_parse::<f32, _>(token).map_err(|e| {
                    LoadMotionError::ParseMotionSection {
                        parse_error: e,
                        channel_index,
                        line: line_num,
                    }
                })?;
                self.motion_values.push(motion);
            }
        }

        if self.motion_values.len() != self.num_channels * self.num_frames {
            return Err(LoadMotionError::MotionCountMismatch {
                actual_total_motion_values: self.motion_values.len(),
                expected_total_motion_values,
                expected_num_frames: self.num_frames,
                expected_num_clips: self.num_channels,
            });
        }

        Ok(())
    }
}
