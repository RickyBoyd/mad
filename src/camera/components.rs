use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct MainCamera;

#[derive(Component, Debug)]
pub struct OrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl OrbitCamera {
    pub fn new(focus: Vec3, position: Vec3) -> Self {
        let offset = position - focus;
        let radius = offset.length();
        let yaw = offset.x.atan2(offset.z);
        let pitch = (offset.y / radius).asin();

        Self {
            focus,
            radius,
            yaw,
            pitch,
        }
    }

    pub fn translation(&self) -> Vec3 {
        let horizontal = self.radius * self.pitch.cos();

        self.focus
            + Vec3::new(
                horizontal * self.yaw.sin(),
                self.radius * self.pitch.sin(),
                horizontal * self.yaw.cos(),
            )
    }
}
