const addEventListeners = () => {
  document.querySelectorAll(".edit-icon").forEach((editIcon) => {
    editIcon.addEventListener("click", function () {
      const editableText = editIcon.closest(".editable-text");
      const textElement = editableText.querySelector(".gallery-title, .caption-text");
      const inputElement = editableText.querySelector(".gallery-title-input, .caption-text-input");
      
      textElement.classList.toggle("hidden");
      inputElement.classList.toggle("hidden");
      if (textElement.classList.contains("hidden")) {
        inputElement.value = textElement.textContent;
        inputElement.focus();
      } else {
        textElement.textContent = inputElement.value;
      }
    });
  });
  
  document.querySelectorAll(".gallery-title-input, .caption-text-input").forEach((input) => {
    input.addEventListener("keyup", function (event) {
      const editableText = input.closest(".editable-text");
      const textElement = editableText.querySelector(".gallery-title, .caption-text");
      
      if (event.key === "Enter") {
        textElement.textContent = input.value;
        input.blur();
        input.classList.toggle("hidden");
        textElement.classList.toggle("hidden");
      }
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

  tippy("[data-tippy-content]", {});
};
