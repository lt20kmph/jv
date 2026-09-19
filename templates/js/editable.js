// Shared editable-text behaviour for gallery names and image captions.
// Loaded by both galleries.js (galleries page) and gallery.js (gallery page).

// Add click listeners only to editable-text containers that have input elements (Writer role)
const addEditableTextListeners = () => {
  document.querySelectorAll(".editable-text:not([data-listeners-added])").forEach((editableText) => {
    const inputElement = editableText.querySelector(
      ".gallery-title-input, .caption-text-input",
    );

    // Only add listeners if input element exists (Writer role)
    if (inputElement) {
      // Mark as having listeners to prevent duplicates
      editableText.setAttribute("data-listeners-added", "true");

      const handler = (event) => {
        // Don't hijack taps on controls that happen to sit inside the tile
        // (e.g. the delete button) - only the text itself toggles edit mode.
        if (event.target.closest("button, a")) return;

        const textElement = editableText.querySelector(
          ".gallery-title, .caption-text",
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
        }
      };

      // A single click listener covers mouse, touch and keyboard activation.
      // The previous click + touchend pair fired twice on some touch browsers,
      // toggling edit mode straight back off.
      editableText.addEventListener("click", handler);
    }
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
      });
    });
};
