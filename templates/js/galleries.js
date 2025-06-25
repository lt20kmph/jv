const addEventListeners = () => {
  // Add click listeners only to editable-text containers that have input elements (Writer role)
  document.querySelectorAll(".editable-text:not([data-listeners-added])").forEach((editableText) => {
    const inputElement = editableText.querySelector(
      ".gallery-title-input, .caption-text-input",
    );
    
    // Only add listeners if input element exists (Writer role)
    if (inputElement) {
      // Mark as having listeners to prevent duplicates
      editableText.setAttribute("data-listeners-added", "true");
      
      editableText.addEventListener("click", function (event) {
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
      });
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
};
