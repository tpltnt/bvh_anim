use atomic_refcell::AtomicRefMut;
use crate::{
    fraction_seconds_to_duration,
    joint::{JointData, JointName},
    Bvh, Channel, ChannelType,
};

#[doc(hidden)]
#[macro_export]
macro_rules! match_channels {
    ($builder:ident; ) => {
        
    };
    ($builder:ident;Xposition $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::PositionX);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Yposition $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::PositionY);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Zposition $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::PositionZ);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Xrotation $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::RotationX);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Yrotation $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::RotationY);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:ident; Zrotation $($rest:ident)*) => {
        $builder.push_channel(bvh_anim::ChannelType::RotationZ);
        bvh_anim::match_channels!($builder; $($rest)*);
    };
    ($builder:expr; $($other:tt)*) => {
        compile_error!("Unknown tokens");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_joints_internal {
    ($builder:ident (
        $(
            JOINT $joint_nm:ident
            {
                $( $children:tt )*
            }
        )*
    )) => {
        $(
            $builder.push_joint();
            $builder.push_joint_name(stringify!($joint_nm));

            $builder.current_depth += 1;
            bvh_anim::parse_joints_internal!($builder ( $( $children )* ));
            $builder.current_depth -= 1;
        )*
    };

    ($builder:ident (
        OFFSET $ofst_x:literal $ofst_y:literal $ofst_z:literal
        $($rest:tt)*
    )) => {
        $builder.push_joint_offset([$ofst_x, $ofst_y, $ofst_z].into(), false);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 0
        $($rest:tt)*
    )) => {
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 1 $ch0:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ; $ch0);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 2 $ch0:ident $ch1:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ; $ch0 $ch1);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 3 $ch0:ident $ch1:ident $ch2:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ; $ch0 $ch1 $ch2);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 4 $ch0:ident $ch1:ident $ch2:ident $ch3:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ; $ch0 $ch1 $ch2 $ch3);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 5
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ; $ch0 $ch1 $ch2 $ch3 $ch4);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 6
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 7
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 8
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
            $ch7:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6 $ch7);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS 9
            $ch0:ident
            $ch1:ident
            $ch2:ident
            $ch3:ident
            $ch4:ident
            $ch5:ident
            $ch6:ident
            $ch7:ident
            $ch8:ident
        $($rest:tt)*
    )) => {
        bvh_anim::match_channels!($builder ;
            $ch0 $ch1 $ch2 $ch3 $ch4 $ch5 $ch6 $ch7 $ch8);
        bvh_anim::parse_joints_internal!($builder ( $($rest)* ));
    };

    ($builder:ident (
        CHANNELS $unsupported:literal $($rest:tt)*
    )) => {
        compile_error!("No more than 9 channels supported in CHANNELS statement");
    };

    ($builder:ident (
        End Site
        {
            OFFSET $end_x:literal $end_y:literal $end_z:literal
        }
    )) => {
        $builder.push_joint_offset([$end_x, $end_y, $end_z].into(), true);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! parse_bvh_internal {
    ($builder:ident (
        HIERARCHY
        MOTION
    )) => {
        $builder.suppress_unused_mut_warning();
    };

    ($builder:ident (
        HIERARCHY
        ROOT $root_name:ident
        {
            $( $joints:tt )*
        }
        MOTION
        Frames: $num_frames:literal
        Frame Time: $frame_time:literal
        $(
            $motion:literal
        )*
    )) => {
        {
            use bvh_anim::parse_joints_internal;

            $builder.push_root();
            $builder.push_joint_name(stringify!($root_name));

            parse_joints_internal!($builder ($($joints)*));

            $builder.set_num_frames($num_frames as usize);
            $builder.set_frame_time(f64::from($frame_time));

            $(
                $builder.push_motion(f32::from($motion));
            )*

            assert!($builder.check_valid_motion());
        }
    };
}

/// Create a new [`Bvh`][`Bvh`] object using a macro literal. Useful for
/// testing.
///
/// # Example
///
/// ```
/// # use bvh_anim::bvh;
/// let simple_skeleton = bvh! {
///     HIERARCHY
///     ROOT Base
///     {
///         OFFSET 0.0 0.0 0.0
///         CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
///         JOINT Middle1
///         {
///             OFFSET 0.0 0.0 15.0
///             CHANNELS 3 Zrotation Xrotation Yrotation
///             JOINT Tip1
///             {
///                 OFFSET 0.0 0.0 30.0
///                 CHANNELS 3 Zrotation Xrotation Yrotation
///                 End Site
///                 {
///                     OFFSET 0.0 0.0 45.0
///                 }
///             }
///         }
///         JOINT Middle2
///         {
///             OFFSET 0.0 15.0 0.0
///             CHANNELS 3 Zrotation Xrotation Yrotation
///             JOINT Tip2
///             {
///                 OFFSET 0.0 30.0 0.0
///                 CHANNELS 3 Zrotation Xrotation Yrotation
///                 End Site
///                 {
///                     OFFSET 0.0 45.0 0.0
///                 }
///             }
///         }
///     }
///
///     MOTION
///     Frames: 1
///     // Time in seconds.
///     Frame Time: 0.033333333333
///     0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
/// };
/// ```
/// 
/// You can combine the `bvh` macro with the `include` macro to include a `bvh` file
/// at compile time:
///
/// ```no_run
/// # use bvh_anim::bvh;
/// let included = bvh! {
///     include!("./path/to/anim.bvh")
/// };
/// ```
/// 
/// You can also use the `bvh` macro to create empty `Bvh` instances:
///
/// ```
/// # use bvh_anim::bvh;
/// let empty = bvh! {};
/// let another_empty = bvh! {
///     HIERARCHY
///     MOTION
/// };
/// ```
///
/// [`bvh`]: struct.Bvh.html
#[macro_export]
macro_rules! bvh {
    ( include!( $file:expr )) => {
        {
            macro_rules! local_include {
                ($file:expr) => {
                    std::include($file)
                }
            }
            bvh! { local_include!($file) }
        }
    };

    () => {
        bvh_anim::Bvh::default()
    };

    ($($toks:tt)*) => {
        {
            use bvh_anim::parse_bvh_internal;
            let mut builder = bvh_anim::BvhLiteralBuilder::default();
            parse_bvh_internal!(builder ($($toks)*));
            builder.bvh
        }
    };
}

/// Helper struct to build a `Bvh` from the macro without exposing
/// too many internals.
#[doc(hidden)]
#[derive(Default)]
pub struct BvhLiteralBuilder {
    pub bvh: Bvh,
    pub current_channel_index: usize,
    pub current_depth: usize,
    pub current_index: usize,
    pub encountered_hierarchy: bool,
    pub encountered_root: bool,
    pub encountered_motion: bool,
    pub encountered_num_frames: bool,
    pub encountered_frame_time: bool,
    pub num_frames: usize,
}

#[doc(hidden)]
impl BvhLiteralBuilder {
    pub fn push_root(&mut self) {
        self.bvh.joints.borrow_mut().push(JointData::empty_root());
        self.current_index += 1;
    }

    pub fn push_joint(&mut self) {
        self.bvh.joints.borrow_mut().push(JointData::empty_child());
        let idx = self.current_index;
        let dpth = self.current_depth;
        {
            let mut private = AtomicRefMut::map(self.last_joint().unwrap(), |j| {
                j.private_data_mut().unwrap()
            });

            private.self_index = idx;
            private.depth = dpth;
            // @TODO: Fix this.
            private.parent_index = 0;
        }

        self.current_index += 1;
    }

    pub fn push_joint_name(&mut self, joint_name: &str) {
        let joint_name = JointName(joint_name.bytes().collect());
        self.last_joint().map(|mut joint| {
            joint.set_name(joint_name);
        });
    }

    pub fn push_channel(&mut self, channel: ChannelType) {
        let channel = Channel::new(channel, self.current_channel_index);
        self.last_joint().map(|mut joint| match *joint {
            JointData::Root {
                ref mut channels, ..
            } => {
                channels.push(channel);
            }
            JointData::Child {
                ref mut channels, ..
            } => {
                channels.push(channel);
            }
        });
        self.current_channel_index += 1;
    }

    pub fn push_joint_offset(&mut self, offset: mint::Vector3<f32>, is_end_site: bool) {
        self.last_joint().map(|mut joint| {
            joint.set_offset(offset, is_end_site);
        });
    }

    #[inline]
    pub fn set_frame_time(&mut self, frame_time_secs: f64) {
        self.bvh
            .clips_mut()
            .set_frame_time(fraction_seconds_to_duration(frame_time_secs));
    }

    #[inline]
    pub fn set_num_frames(&mut self, num_frames: usize) {
        self.num_frames = num_frames;
        let mut clips = self.bvh.clips.borrow_mut();
        clips.num_channels = self.current_channel_index;
        clips.num_frames = self.num_frames;
        clips
            .data
            .reserve(self.current_channel_index * self.num_frames);
    }

    #[inline]
    pub fn push_motion(&mut self, motion: f32) {
        let mut clips = self.bvh.clips.borrow_mut();
        clips.data.push(motion);
    }

    pub fn check_valid_motion(&self) -> bool {
        let clips = self.bvh.clips.borrow();
        clips.data.len() == clips.num_channels * clips.num_frames
    }

    pub fn suppress_unused_mut_warning(&mut self) {}

    fn last_joint(&mut self) -> Option<AtomicRefMut<'_, JointData>> {
        let joints = self.bvh.joints.borrow_mut();
        if joints.is_empty() {
            None
        } else {
            Some(AtomicRefMut::map(joints, |j| j.last_mut().unwrap()))
        }
    }
}

#[cfg(test)]
mod tests {
    // Needed for macros
    use crate as bvh_anim;
    use std::time::Duration;

    #[test]
    fn test_macro() {
        let bvh = bvh! {
            HIERARCHY
            ROOT Base
            {
                OFFSET 0.0 0.0 0.0
                CHANNELS 6 Xposition Yposition Zposition Zrotation Xrotation Yrotation
                JOINT End
                {
                    OFFSET 0.0 0.0 15.0
                    CHANNELS 3 Zrotation Xrotation Yrotation
                    End Site
                    {
                        OFFSET 0.0 0.0 30.0
                    }
                }
            }

            MOTION
            Frames: 1
            Frame Time: 0.033333333333
            0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0 0.0
        };

        // @TODO: test joints

        assert_eq!(*bvh.clips().frame_time(), Duration::from_nanos(33333333));

        let mut num_frames = 0;
        for frame in bvh.clips().frames() {
            let mut num_channels = 0;
            for channel in frame.iter() {
                assert_eq!(*channel, 0.0);
                num_channels += 1;
            }

            assert_eq!(num_channels, 9);
            num_frames += 1;
        }
        assert_eq!(num_frames, 1);
    }

    #[test]
    fn test_empty_create() {
        macro_rules! assert_empty {
            ($bvh:expr) => {
                assert!($bvh.joints().next().is_none());
                assert_eq!(*$bvh.clips().frame_time(), Duration::default());
                assert!($bvh.clips().frames().next().is_none());
            };
        }

        let empty = bvh!{};
        assert_empty!(empty);

        let empty_2 = bvh! {
            HIERARCHY
            MOTION
        };
        assert_empty!(empty_2);
    }
}