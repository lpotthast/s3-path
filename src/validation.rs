use crate::error::InvalidS3PathComponent;

/// Validates that a path component contains only allowed characters:
/// alphanumeric characters, hyphens, underscores, and periods.
pub(crate) fn validate_component(component: &str) -> Result<(), InvalidS3PathComponent> {
    if component.is_empty() {
        return Err(InvalidS3PathComponent {
            component: component.to_string(),
            reason: "Empty component is not allowed".to_string(),
        });
    }

    for c in component.chars() {
        if !c.is_alphanumeric() && c != '-' && c != '_' && c != '.' {
            return Err(InvalidS3PathComponent {
                component: component.to_string(),
                reason: format!("Character '{}' is not allowed", c),
            });
        }
    }

    Ok(())
}
