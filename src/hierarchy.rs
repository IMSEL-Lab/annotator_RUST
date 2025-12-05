use crate::classes::{ClassConfig, HierarchicalClassNode};

/// Navigation state for hierarchical class selection
#[derive(Debug, Clone)]
pub struct HierarchyNavigator {
    /// Current path through the hierarchy (keys pressed)
    pub path: Vec<u8>,
    /// The full hierarchy tree
    hierarchy: Vec<HierarchicalClassNode>,
    /// Max depth of the hierarchy (1=flat, 2=two-tier, 3=three-tier)
    max_depth: usize,
}

impl HierarchyNavigator {
    /// Create a new navigator from a class config
    pub fn new(config: &ClassConfig) -> Self {
        let max_depth = validate_and_get_depth(&config.hierarchy);

        Self {
            path: Vec::new(),
            hierarchy: config.hierarchy.clone(),
            max_depth,
        }
    }

    /// Check if hierarchy mode is active (more than 5 classes)
    pub fn is_hierarchical(&self) -> bool {
        !self.hierarchy.is_empty()
    }

    /// Get current depth in the tree
    pub fn current_depth(&self) -> usize {
        self.path.len()
    }

    /// Navigate down by pressing a key (1-5)
    /// Returns Some(class_id) if we reached a leaf, None if we just navigated deeper
    pub fn navigate_down(&mut self, key: u8) -> Option<i32> {
        if key < 1 || key > 5 {
            return None;
        }

        // Get current level nodes
        let current_nodes = self.get_current_level_nodes();

        // Find node with matching key and clone its data
        let node_data = current_nodes.iter()
            .find(|n| n.key == key)
            .map(|n| (n.id, n.children.is_empty()))?;

        // Add key to path
        self.path.push(key);

        // If this is a leaf node, return the class ID and reset
        if let (Some(id), _) = node_data {
            self.reset();
            Some(id)
        } else {
            // Continue navigation
            None
        }
    }

    /// Navigate up one level (ESC/Backspace)
    pub fn navigate_up(&mut self) {
        self.path.pop();
    }

    /// Reset to root
    pub fn reset(&mut self) {
        self.path.clear();
    }

    /// Get the nodes at the current navigation level
    pub fn get_current_level_nodes(&self) -> Vec<&HierarchicalClassNode> {
        let mut current = &self.hierarchy;

        for &key in &self.path {
            // Find the child node with this key
            if let Some(node) = current.iter().find(|n| n.key == key) {
                current = &node.children;
            } else {
                // Invalid path, return empty
                return Vec::new();
            }
        }

        current.iter().collect()
    }

    /// Get breadcrumb path for UI display
    /// Returns something like ["Living", "Pets & Farm"] for path [1, 2]
    pub fn get_breadcrumb(&self) -> Vec<String> {
        let mut breadcrumb = Vec::new();
        let mut current = &self.hierarchy;

        for &key in &self.path {
            if let Some(node) = current.iter().find(|n| n.key == key) {
                breadcrumb.push(node.label.clone());
                current = &node.children;
            }
        }

        breadcrumb
    }

    /// Get prompt text for current level
    pub fn get_prompt(&self) -> String {
        if self.path.is_empty() {
            "Select category (1-5)".to_string()
        } else if self.current_depth() < self.max_depth {
            "Select subcategory (1-5)".to_string()
        } else {
            "Select class (1-5)".to_string()
        }
    }

    /// Check if at root level
    #[allow(dead_code)]
    pub fn is_at_root(&self) -> bool {
        self.path.is_empty()
    }

    /// Get max depth of hierarchy
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

/// Validate hierarchy structure and determine depth
/// Returns 0 if flat (no hierarchy), 1-3 for hierarchical depth
fn validate_and_get_depth(nodes: &[HierarchicalClassNode]) -> usize {
    if nodes.is_empty() {
        return 0;
    }

    let mut max_depth = 1;

    for node in nodes {
        if !node.children.is_empty() {
            let child_depth = validate_and_get_depth(&node.children);
            max_depth = max_depth.max(child_depth + 1);
        }
    }

    max_depth
}

/// Validate that hierarchy meets constraints
#[allow(dead_code)]
pub fn validate_hierarchy(nodes: &[HierarchicalClassNode]) -> Result<(), String> {
    // Check root level has at most 5 nodes
    if nodes.len() > 5 {
        return Err(format!("Root level has {} nodes, max 5 allowed", nodes.len()));
    }

    // Recursively validate children
    validate_hierarchy_recursive(nodes, 1)?;

    // Check max depth
    let depth = validate_and_get_depth(nodes);
    if depth > 3 {
        return Err(format!("Hierarchy depth is {}, max 3 allowed", depth));
    }

    Ok(())
}

#[allow(dead_code)]
fn validate_hierarchy_recursive(nodes: &[HierarchicalClassNode], level: usize) -> Result<(), String> {
    if nodes.len() > 5 {
        return Err(format!("Level {} has {} nodes, max 5 allowed", level, nodes.len()));
    }

    // Check all keys are 1-5
    for node in nodes {
        if node.key < 1 || node.key > 5 {
            return Err(format!("Invalid key {} at level {}, must be 1-5", node.key, level));
        }

        // Validate children
        if !node.children.is_empty() {
            validate_hierarchy_recursive(&node.children, level + 1)?;
        }
    }

    Ok(())
}

/// Count total leaf classes in hierarchy
#[allow(dead_code)]
pub fn count_leaf_classes(nodes: &[HierarchicalClassNode]) -> usize {
    let mut count = 0;

    for node in nodes {
        if node.id.is_some() {
            count += 1;
        }
        count += count_leaf_classes(&node.children);
    }

    count
}

/// Determine required hierarchy depth based on class count
#[allow(dead_code)]
pub fn required_hierarchy_depth(class_count: usize) -> Result<usize, String> {
    match class_count {
        0 => Err("No classes defined".to_string()),
        1..=5 => Ok(1),    // Flat mode
        6..=25 => Ok(2),   // 2-tier mode
        26..=125 => Ok(3), // 3-tier mode
        _ => Err(format!("Too many classes ({}), max 125 supported", class_count)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_depth() {
        assert_eq!(required_hierarchy_depth(3).unwrap(), 1);
        assert_eq!(required_hierarchy_depth(5).unwrap(), 1);
        assert_eq!(required_hierarchy_depth(6).unwrap(), 2);
        assert_eq!(required_hierarchy_depth(25).unwrap(), 2);
        assert_eq!(required_hierarchy_depth(26).unwrap(), 3);
        assert_eq!(required_hierarchy_depth(125).unwrap(), 3);
        assert!(required_hierarchy_depth(126).is_err());
    }
}
