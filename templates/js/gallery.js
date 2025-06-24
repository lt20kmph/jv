function beforeUploadFormRequest() {
  if (document.getElementById("upload_form").innerHTML) {
    event.preventDefault();
    document.getElementById("upload_form").style.display = "block";
  }
}

function updateGalleryEmptyState() {
  const gallery = document.getElementById("gallery");
  const emptyMessage = gallery.querySelector("p.empty-message");

  // Count actual image items (exclude any existing empty message)
  const imageItems = gallery.querySelectorAll(".unified-tile");

  if (imageItems.length === 0) {
    // Gallery is empty, show message if not already present
    if (!emptyMessage) {
      const message = document.createElement("p");
      message.textContent = "Hmm... Nothing here yet...";
      message.className = "empty-message";
      gallery.appendChild(message);
    }
  } else {
    // Gallery has images, remove message if present
    if (emptyMessage) {
      emptyMessage.remove();
    }
  }
}

// Add event listeners for editable text (same as galleries.js)
const addEditableTextListeners = () => {
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

      // Add blur event listener to handle clicking outside
      input.addEventListener("blur", function (event) {
        const editableText = input.closest(".editable-text");
        const textElement = editableText.querySelector(
          ".gallery-title, .caption-text",
        );

        // Exit edit mode when clicking outside
        textElement.textContent = input.value;
        input.classList.add("hidden");
        textElement.classList.remove("hidden");
        // Update tooltip after text change
        updateTooltips();
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

// Initialize empty state on page load and set up observer
document.addEventListener("DOMContentLoaded", function () {
  updateGalleryEmptyState();
  addEditableTextListeners();

  // Set up MutationObserver to watch for changes in the gallery
  const gallery = document.getElementById("gallery");
  if (gallery) {
    const observer = new MutationObserver(function (mutations) {
      // Check if any mutations involved adding or removing child nodes
      const hasChildListMutation = mutations.some(
        (mutation) =>
          mutation.type === "childList" &&
          (mutation.addedNodes.length > 0 || mutation.removedNodes.length > 0),
      );

      if (hasChildListMutation) {
        updateGalleryEmptyState();
        // Re-add event listeners for any new editable elements
        addEditableTextListeners();
      }
    });

    // Start observing child list changes
    observer.observe(gallery, {
      childList: true,
      subtree: true,
    });
  }
});
