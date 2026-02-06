function setupDirtyForms() {
  const forms = document.querySelectorAll("form");
  let anyDirty = false;

  forms.forEach((form) => {
    const submitButton: HTMLButtonElement | null = form.querySelector(
      'button[type="submit"]',
    );
    const anyFieldRequired = form.querySelector("[required]") !== null;

    if (!submitButton || !anyFieldRequired) {
      return;
    }

    let isDirty = false;

    const updateSubmitButtons = () => {
      if (isDirty && form.checkValidity()) {
        submitButton.disabled = false;
      } else {
        submitButton.disabled = true;
      }
    };

    form.addEventListener("input", () => {
      isDirty = true;
      anyDirty = true;
      updateSubmitButtons();
    });

    form.addEventListener("change", () => {
      isDirty = true;
      anyDirty = true;
      updateSubmitButtons();
    });

    updateSubmitButtons();
  });

  window.addEventListener("beforeunload", (event) => {
    if (anyDirty) {
      event.preventDefault();
    }
  });
}

if (typeof window !== "undefined") {
  window.addEventListener("load", setupDirtyForms);
}
