//! Light linking system for controlling which lights affect which objects.
//!
//! This module provides a flexible system for controlling light-object interactions
//! using bit masks and channels. This allows fine-grained control over which lights
//! illuminate which objects, essential for complex scenes and artistic control.
//!
//! # Architecture
//!
//! - **Light Channels**: 32-bit masks for light grouping
//! - **Object Masks**: 32-bit masks for object light reception
//! - **Bitwise Operations**: Fast GPU-side filtering
//! - **Dynamic Updates**: Real-time channel modification
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::{LightLinkingManager, LightChannel};
//!
//! # fn example() -> praxis_utils::Result<()> {
//! let mut manager = LightLinkingManager::new();
//!
//! // Define channels
//! let hero_lights = 0b0001;
//! let environment_lights = 0b0010;
//! let effect_lights = 0b0100;
//!
//! // Configure objects
//! manager.set_object_mask("hero", hero_lights | environment_lights)?;
//! manager.set_object_mask("background", environment_lights)?;
//!
//! // Configure lights
//! manager.set_light_channel("key_light", hero_lights)?;
//! manager.set_light_channel("ambient", environment_lights)?;
//! # Ok(())
//! # }
//! ```

use praxis_utils::Result;
use std::collections::HashMap;

/// Default light channel (all lights affect all objects).
pub const DEFAULT_LIGHT_CHANNEL: u32 = 0xFFFFFFFF;

/// Light channel identifier (32-bit mask).
pub type LightChannel = u32;

/// Light linking mask for an object or light.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightLinkingMask {
    pub mask: u32,
    pub _padding: [u32; 3],
}

impl LightLinkingMask {
    pub fn new(mask: u32) -> Self {
        Self {
            mask,
            _padding: [0; 3],
        }
    }

    pub fn all() -> Self {
        Self::new(DEFAULT_LIGHT_CHANNEL)
    }

    pub fn none() -> Self {
        Self::new(0)
    }

    pub fn channel(channel: LightChannel) -> Self {
        Self::new(1 << channel)
    }

    pub fn channels(channels: &[LightChannel]) -> Self {
        let mut mask = 0u32;
        for &channel in channels {
            mask |= 1 << channel;
        }
        Self::new(mask)
    }

    pub fn includes(&self, other: &Self) -> bool {
        (self.mask & other.mask) != 0
    }

    pub fn add_channel(&mut self, channel: LightChannel) {
        self.mask |= 1 << channel;
    }

    pub fn remove_channel(&mut self, channel: LightChannel) {
        self.mask &= !(1 << channel);
    }

    pub fn toggle_channel(&mut self, channel: LightChannel) {
        self.mask ^= 1 << channel;
    }

    pub fn has_channel(&self, channel: LightChannel) -> bool {
        (self.mask & (1 << channel)) != 0
    }
}

impl Default for LightLinkingMask {
    fn default() -> Self {
        Self::all()
    }
}

impl From<u32> for LightLinkingMask {
    fn from(mask: u32) -> Self {
        Self::new(mask)
    }
}

impl From<LightLinkingMask> for u32 {
    fn from(mask: LightLinkingMask) -> Self {
        mask.mask
    }
}

/// Manager for light linking system.
pub struct LightLinkingManager {
    object_masks: HashMap<String, LightLinkingMask>,
    light_channels: HashMap<String, LightChannel>,
    channel_names: HashMap<LightChannel, String>,
}

impl LightLinkingManager {
    pub fn new() -> Self {
        Self {
            object_masks: HashMap::new(),
            light_channels: HashMap::new(),
            channel_names: HashMap::new(),
        }
    }

    pub fn register_channel(&mut self, channel: LightChannel, name: String) {
        self.channel_names.insert(channel, name);
    }

    pub fn get_channel_by_name(&self, name: &str) -> Option<LightChannel> {
        self.channel_names
            .iter()
            .find(|(_, n)| n.as_str() == name)
            .map(|(c, _)| *c)
    }

    pub fn set_object_mask(&mut self, object_id: &str, mask: u32) -> Result<()> {
        self.object_masks
            .insert(object_id.to_string(), LightLinkingMask::new(mask));
        Ok(())
    }

    pub fn get_object_mask(&self, object_id: &str) -> LightLinkingMask {
        self.object_masks
            .get(object_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn add_object_channel(&mut self, object_id: &str, channel: LightChannel) -> Result<()> {
        let mask = self.object_masks.entry(object_id.to_string()).or_default();
        mask.add_channel(channel);
        Ok(())
    }

    pub fn remove_object_channel(&mut self, object_id: &str, channel: LightChannel) -> Result<()> {
        if let Some(mask) = self.object_masks.get_mut(object_id) {
            mask.remove_channel(channel);
        }
        Ok(())
    }

    pub fn set_light_channel(&mut self, light_id: &str, channel: LightChannel) -> Result<()> {
        self.light_channels.insert(light_id.to_string(), channel);
        Ok(())
    }

    pub fn get_light_channel(&self, light_id: &str) -> LightChannel {
        self.light_channels
            .get(light_id)
            .copied()
            .unwrap_or(DEFAULT_LIGHT_CHANNEL)
    }

    pub fn can_light_affect_object(&self, light_id: &str, object_id: &str) -> bool {
        let light_channel = self.get_light_channel(light_id);
        let object_mask = self.get_object_mask(object_id);
        object_mask.has_channel(light_channel)
    }

    pub fn clear_object(&mut self, object_id: &str) {
        self.object_masks.remove(object_id);
    }

    pub fn clear_light(&mut self, light_id: &str) {
        self.light_channels.remove(light_id);
    }

    pub fn clear_all(&mut self) {
        self.object_masks.clear();
        self.light_channels.clear();
    }

    pub fn list_objects(&self) -> Vec<&str> {
        self.object_masks.keys().map(|s| s.as_str()).collect()
    }

    pub fn list_lights(&self) -> Vec<&str> {
        self.light_channels.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for LightLinkingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_linking_mask_creation() {
        let mask = LightLinkingMask::new(0b1010);
        assert_eq!(mask.mask, 0b1010);
    }

    #[test]
    fn test_light_linking_mask_all() {
        let mask = LightLinkingMask::all();
        assert_eq!(mask.mask, DEFAULT_LIGHT_CHANNEL);
    }

    #[test]
    fn test_light_linking_mask_none() {
        let mask = LightLinkingMask::none();
        assert_eq!(mask.mask, 0);
    }

    #[test]
    fn test_light_linking_mask_channel() {
        let mask = LightLinkingMask::channel(3);
        assert_eq!(mask.mask, 0b1000);
    }

    #[test]
    fn test_light_linking_mask_channels() {
        let mask = LightLinkingMask::channels(&[0, 2, 4]);
        assert_eq!(mask.mask, 0b10101);
    }

    #[test]
    fn test_light_linking_mask_includes() {
        let mask1 = LightLinkingMask::new(0b1010);
        let mask2 = LightLinkingMask::new(0b0010);
        let mask3 = LightLinkingMask::new(0b0100);

        assert!(mask1.includes(&mask2));
        assert!(!mask1.includes(&mask3));
    }

    #[test]
    fn test_light_linking_mask_add_channel() {
        let mut mask = LightLinkingMask::new(0b0001);
        mask.add_channel(2);
        assert_eq!(mask.mask, 0b0101);
    }

    #[test]
    fn test_light_linking_mask_remove_channel() {
        let mut mask = LightLinkingMask::new(0b0101);
        mask.remove_channel(2);
        assert_eq!(mask.mask, 0b0001);
    }

    #[test]
    fn test_light_linking_mask_toggle_channel() {
        let mut mask = LightLinkingMask::new(0b0101);
        mask.toggle_channel(0);
        assert_eq!(mask.mask, 0b0100);
        mask.toggle_channel(0);
        assert_eq!(mask.mask, 0b0101);
    }

    #[test]
    fn test_light_linking_mask_has_channel() {
        let mask = LightLinkingMask::new(0b1010);
        assert!(!mask.has_channel(0));
        assert!(mask.has_channel(1));
        assert!(!mask.has_channel(2));
        assert!(mask.has_channel(3));
    }

    #[test]
    fn test_light_linking_manager_basic() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0001).unwrap();
        manager.set_light_channel("light1", 0).unwrap();

        assert!(manager.can_light_affect_object("light1", "obj1"));
    }

    #[test]
    fn test_light_linking_manager_multiple_channels() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0011).unwrap();
        manager.set_light_channel("light1", 0).unwrap();
        manager.set_light_channel("light2", 1).unwrap();
        manager.set_light_channel("light3", 2).unwrap();

        assert!(manager.can_light_affect_object("light1", "obj1"));
        assert!(manager.can_light_affect_object("light2", "obj1"));
        assert!(!manager.can_light_affect_object("light3", "obj1"));
    }

    #[test]
    fn test_light_linking_manager_add_remove_channels() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0001).unwrap();
        manager.add_object_channel("obj1", 1).unwrap();

        let mask = manager.get_object_mask("obj1");
        assert_eq!(mask.mask, 0b0011);

        manager.remove_object_channel("obj1", 0).unwrap();
        let mask = manager.get_object_mask("obj1");
        assert_eq!(mask.mask, 0b0010);
    }

    #[test]
    fn test_light_linking_manager_channel_names() {
        let mut manager = LightLinkingManager::new();

        manager.register_channel(0, "hero".to_string());
        manager.register_channel(1, "ambient".to_string());

        assert_eq!(manager.get_channel_by_name("hero"), Some(0));
        assert_eq!(manager.get_channel_by_name("ambient"), Some(1));
        assert_eq!(manager.get_channel_by_name("nonexistent"), None);
    }

    #[test]
    fn test_light_linking_manager_clear() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0001).unwrap();
        manager.set_light_channel("light1", 0).unwrap();

        manager.clear_object("obj1");
        assert_eq!(manager.get_object_mask("obj1"), LightLinkingMask::all());

        manager.clear_light("light1");
        assert_eq!(manager.get_light_channel("light1"), DEFAULT_LIGHT_CHANNEL);
    }

    #[test]
    fn test_light_linking_manager_list() {
        let mut manager = LightLinkingManager::new();

        manager.set_object_mask("obj1", 0b0001).unwrap();
        manager.set_object_mask("obj2", 0b0010).unwrap();
        manager.set_light_channel("light1", 0).unwrap();

        let objects = manager.list_objects();
        assert_eq!(objects.len(), 2);
        assert!(objects.contains(&"obj1"));
        assert!(objects.contains(&"obj2"));

        let lights = manager.list_lights();
        assert_eq!(lights.len(), 1);
        assert!(lights.contains(&"light1"));
    }
}
