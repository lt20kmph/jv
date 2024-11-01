document.getElementById('signup-form').addEventListener('input', function () {
  validateForm();
});

function validateForm() {
  const email = document.getElementById('signup-form-email').value;
  const password = document.getElementById('signup-form-password').value;
  const confirmPassword = document.getElementById('signup-form-password-repeat').value;
  const submitBtn = document.getElementById('signup-form').querySelector('button');
  const errorElement = document.getElementById('password-error');

  let isValid = true;

  if (!email || !password || !confirmPassword) {
    isValid = false;
  }

  if (password !== confirmPassword) {
    errorElement.textContent = 'Passwords do not match';
    errorElement.classList.remove('success');
    errorElement.classList.add('error');
    isValid = false;
  } else {
    errorElement.textContent = 'Passwords match';
    errorElement.classList.remove('error');
    errorElement.classList.add('success');
  }

  if (isValid) {
    submitBtn.classList.add('enabled');
    submitBtn.disabled = false;
  } else {
    submitBtn.classList.remove('enabled');
    submitBtn.disabled = true;
  }
}
