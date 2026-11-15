import { createElement } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import * as AlertDialog from "@radix-ui/react-alert-dialog";

// A styled, controlled Radix modal. Callers own application state; no emulated
// HTMLDialogElement or imperative show/close API is exposed.
export function ModalRoot({
  opened,
  onOpenChange,
  canDismiss = true,
  alert = false,
  trigger,
  children,
}) {
  const Primitive = alert ? AlertDialog : Dialog;
  return createElement(
    Primitive.Root,
    {
      open: opened,
      onOpenChange: (next) => {
        if (next || canDismiss) onOpenChange(next);
      },
    },
    trigger && createElement(Primitive.Trigger, { asChild: true }, trigger),
    children,
  );
}

export function Modal(props) {
  return createElement(ModalRoot, props, createElement(ModalContent, props));
}

export function ModalContent({
  title,
  canDismiss = true,
  alert = false,
  id,
  onKeyDown,
  className = "",
  style,
  contentRef,
  onOpenAutoFocus,
  onCloseAutoFocus,
  children,
}) {
  const Primitive = alert ? AlertDialog : Dialog;
  return createElement(
    Primitive.Portal,
    null,
    createElement(
      "div",
      { className: "modal-layer" },
      createElement(Primitive.Overlay, { className: "modal-overlay" }),
      createElement(
        Primitive.Content,
        {
          id,
          onKeyDown,
          className: `modal-content ${className}`,
          style,
          ref: contentRef,
          "data-ui-modal": "",
          "aria-describedby": undefined,
          onEscapeKeyDown: (event) => {
            event.stopPropagation();
            if (!canDismiss) event.preventDefault();
          },
          onPointerDownOutside: (event) => {
            if (!canDismiss) event.preventDefault();
          },
          onOpenAutoFocus,
          onCloseAutoFocus,
        },
        createElement(
          Primitive.Title,
          { asChild: true },
          createElement("span", { hidden: true }, title),
        ),
        children,
      ),
    ),
  );
}
