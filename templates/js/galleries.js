const addEventListeners = () => {
  // Add click listeners only to editable-text containers that don't already have listeners
  document.querySelectorAll(".editable-text:not([data-listeners-added])").forEach((editableText) => {
    // Mark as having listeners to prevent duplicates
    editableText.setAttribute("data-listeners-added", "true");
    
    editableText.addEventListener("click", function (event) {
      const textElement = editableText.querySelector(
        ".gallery-title, .caption-text",
      );
      const inputElement = editableText.querySelector(
        ".gallery-title-input, .caption-text-input",
      );

      // Prevent multiple event handlers from firing
      event.stopPropagation();

      textElement.classList.toggle("hidden");
      inputElement.classList.toggle("hidden");
      if (textElement.classList.contains("hidden")) {
        inputElement.value = textElement.textContent;
        inputElement.focus();
      } else {
        textElement.textContent = inputElement.value;
        // Update tooltip after text change
        updateTooltips();
      }
    });
  });

  document
    .querySelectorAll(".gallery-title-input:not([data-keyup-listener]), .caption-text-input:not([data-keyup-listener])")
    .forEach((input) => {
      // Mark as having keyup listener to prevent duplicates
      input.setAttribute("data-keyup-listener", "true");
      
      input.addEventListener("keyup", function (event) {
        const editableText = input.closest(".editable-text");
        const textElement = editableText.querySelector(
          ".gallery-title, .caption-text",
        );

        if (event.key === "Enter") {
          textElement.textContent = input.value;
          input.blur();
          input.classList.toggle("hidden");
          textElement.classList.toggle("hidden");
          // Update tooltip after text change
          updateTooltips();
        }
      });
    });

  // Initialize tooltips for editable text elements
  updateTooltips();
};

const updateTooltips = () => {
  // Add tooltips to editable text elements that have overflow
  document.querySelectorAll(".editable-text").forEach((editableElement) => {
    const textElement = editableElement.querySelector(
      ".gallery-title, .caption-text",
    );
    if (textElement) {
      const isOverflowing = textElement.scrollWidth > textElement.clientWidth;

      if (isOverflowing) {
        // Add tooltip with full text if truncated
        textElement.setAttribute("data-tippy-content", textElement.textContent);
        // Also add edit instruction
        const fullTooltip = `${textElement.textContent}\n\nClick to edit`;
        textElement.setAttribute("data-tippy-content", fullTooltip);
      } else {
        // Just show edit instruction
        textElement.setAttribute("data-tippy-content", "Click to edit");
      }
    }
  });

  // Reinitialize tippy for new elements
  tippy("[data-tippy-content]", {
    allowHTML: false,
    theme: "light-border",
    placement: "top",
  });
};

window.onload = function () {
  // Select the parent element to observe
  const parentElement = document.getElementById("galleries");

  // Create a new MutationObserver instance
  const observer = new MutationObserver((mutationsList) => {
    for (const mutation of mutationsList) {
      if (mutation.type === "childList") {
        addEventListeners();
      }
    }
  });

  // Configure the observer to watch for child node additions
  const config = { childList: true };

  // Start observing the parent element
  observer.observe(parentElement, config);

  addEventListeners();

  tippy("[data-tippy-content]", {});
};
