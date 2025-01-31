const addEventListeners = () => {
  document.querySelectorAll(".edit-icon").forEach((editIcon) => {
    editIcon.addEventListener("click", function () {
      const title = editIcon
        .closest(".editable-title")
        .querySelector(".gallery-title");
      const input = editIcon
        .closest(".editable-title")
        .querySelector(".gallery-title-input");
      title.classList.toggle("hidden");
      input.classList.toggle("hidden");
      if (title.classList.contains("hidden")) {
        input.value = title.textContent;
        input.focus();
      } else {
        title.textContent = input.value;
      }
    });
  });
  document.querySelectorAll(".gallery-title-input").forEach((input) => {
    input.addEventListener("keyup", function (event) {
      const title = input
        .closest(".editable-title")
        .querySelector(".gallery-title");
      if (event.key === "Enter") {
        title.textContent = input.value;
        input.blur();
        input.classList.toggle("hidden");
        title.classList.toggle("hidden");
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
