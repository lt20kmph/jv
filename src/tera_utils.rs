use crate::constants;
use log::error;
use tera::Context;

/// Renders a Tera template with comprehensive error logging
/// 
/// This function wraps the standard Tera template rendering with detailed error logging
/// to help with debugging template issues. It logs the template name and provides
/// context about what went wrong during rendering.
/// 
/// # Arguments
/// * `template_name` - The name of the template file to render
/// * `context` - The Tera context containing variables for the template
/// 
/// # Returns
/// * `Result<String, tera::Error>` - The rendered template string or a Tera error
/// 
/// # Example
/// ```rust
/// let mut context = tera::Context::new();
/// context.insert("title", "My Page");
/// let html = render_template_with_logging("index.html", &context)?;
/// ```
pub fn render_template_with_logging(
    template_name: &str,
    context: &Context,
) -> Result<String, tera::Error> {
    match constants::TEMPLATES.render(template_name, context) {
        Ok(rendered) => {
            // Optionally log successful renders at debug level
            log::debug!("Successfully rendered template: {}", template_name);
            Ok(rendered)
        }
        Err(e) => {
            // Log detailed error information
            error!(
                "Template render error for '{}': {:?}",
                template_name, e
            );
            
            // Return the original error for proper error handling
            Err(e)
        }
    }
}