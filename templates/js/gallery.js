function beforeUploadFormRequest() {
  if (document.getElementById("upload_form").innerHTML) {
    event.preventDefault();
    document.getElementById("upload_form").style.display = "block";
  }
}

// Lightbox controls

function closeLightbox() {
  document.getElementById("lightbox").innerHTML = "";
}

function setupLightboxListeners() {
  const lightboxTarget = document.getElementById("lightbox");
  if (!lightboxTarget) return;

  // Click on the backdrop (outside the image) closes the lightbox
  lightboxTarget.addEventListener("click", (event) => {
    if (event.target === lightboxTarget) closeLightbox();
  });

  let touchStartX = null;

  // Swipe left/right navigates to the previous/next image
  document.addEventListener(
    "touchstart",
    (event) => {
      const touch = event.touches[0];
      if (
        touch &&
        touch.target instanceof Element &&
        touch.target.closest(".lightbox")
      ) {
        touchStartX = touch.clientX;
      }
    },
    { passive: true },
  );

  document.addEventListener(
    "touchend",
    (event) => {
      if (touchStartX === null) return;

      const deltaX = event.changedTouches[0].clientX - touchStartX;
      touchStartX = null;

      if (Math.abs(deltaX) < 60) return;

      const button = deltaX < 0
        ? document.querySelector(".lightbox-next-button")
        : document.querySelector(".lightbox-prev-button");
      if (button) button.click();
    },
    { passive: true },
  );
}

// Escape closes the lightbox
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeLightbox();
});

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

// Initialize empty state on page load and set up observer
document.addEventListener("DOMContentLoaded", function () {
  updateGalleryEmptyState();
  addEditableTextListeners();
  setupLightboxListeners();

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
