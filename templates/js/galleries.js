// Galleries page: re-bind editable-text listeners as new tiles are added.

window.onload = function () {
  // Select the parent element to observe
  const parentElement = document.getElementById("galleries");

  // Create a new MutationObserver instance
  const observer = new MutationObserver((mutationsList) => {
    for (const mutation of mutationsList) {
      if (mutation.type === "childList") {
        addEditableTextListeners();
      }
    }
  });

  // Configure the observer to watch for child node additions
  const config = { childList: true };

  // Start observing the parent element
  observer.observe(parentElement, config);

  addEditableTextListeners();
};
