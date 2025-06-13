function beforeUploadFormRequest() {
  if (document.getElementById("upload_form").innerHTML) {
    event.preventDefault();
    document.getElementById("upload_form").style.display = "block";
  }
}

function updateGalleryEmptyState() {
  const gallery = document.getElementById('gallery');
  const emptyMessage = gallery.querySelector('p.empty-message');
  
  // Count actual image items (exclude any existing empty message)
  const imageItems = gallery.querySelectorAll('.unified-tile');
  
  if (imageItems.length === 0) {
    // Gallery is empty, show message if not already present
    if (!emptyMessage) {
      const message = document.createElement('p');
      message.textContent = 'Hmm... Nothing here yet...';
      message.className = 'empty-message';
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
document.addEventListener('DOMContentLoaded', function() {
  updateGalleryEmptyState();
  
  // Set up MutationObserver to watch for changes in the gallery
  const gallery = document.getElementById('gallery');
  if (gallery) {
    const observer = new MutationObserver(function(mutations) {
      // Check if any mutations involved adding or removing child nodes
      const hasChildListMutation = mutations.some(mutation => 
        mutation.type === 'childList' && 
        (mutation.addedNodes.length > 0 || mutation.removedNodes.length > 0)
      );
      
      if (hasChildListMutation) {
        updateGalleryEmptyState();
      }
    });
    
    // Start observing child list changes
    observer.observe(gallery, {
      childList: true,
      subtree: true
    });
  }
});
