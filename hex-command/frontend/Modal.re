type focus_event;
[@mel.send] external preventDefault: focus_event => unit = "preventDefault";
module Primitive = {
  [@react.component] [@mel.module {js|@presentation/modal.mjs|js}]
  external make:
    (
      ~opened: bool,
      ~onOpenChange: bool => unit,
      ~title: string,
      ~id: string=?,
      ~onKeyDown: React.Event.Keyboard.t => unit=?,
      ~canDismiss: bool=?,
      ~alert: bool=?,
      ~className: string=?,
      ~style: ReactDOM.Style.t=?,
      ~contentRef: React.ref(Js.Nullable.t(Dom.element))=?,
      ~onOpenAutoFocus: focus_event => unit=?,
      ~onCloseAutoFocus: focus_event => unit=?,
      ~trigger: React.element=?,
      ~children: React.element
    ) =>
    React.element =
    "Modal";
};
[@react.component]
let make =
    (
      ~opened,
      ~onOpenChange,
      ~title,
      ~id=?,
      ~onKeyDown=?,
      ~canDismiss=true,
      ~alert=false,
      ~className={js||js},
      ~style=?,
      ~contentRef=?,
      ~onOpenAutoFocus=?,
      ~onCloseAutoFocus=?,
      ~trigger=?,
      ~children,
    ) =>
  <Primitive
    opened
    onOpenChange
    title
    ?id
    ?onKeyDown
    canDismiss
    alert
    className
    ?style
    ?contentRef
    ?onOpenAutoFocus
    ?onCloseAutoFocus
    ?trigger>
    children
  </Primitive>;

module Root = {
  [@react.component] [@mel.module {js|@presentation/modal.mjs|js}]
  external make:
    (
      ~opened: bool,
      ~onOpenChange: bool => unit,
      ~canDismiss: bool=?,
      ~alert: bool=?,
      ~trigger: React.element=?,
      ~children: React.element
    ) =>
    React.element =
    "ModalRoot";
};
module Content = {
  [@react.component] [@mel.module {js|@presentation/modal.mjs|js}]
  external make:
    (
      ~title: string,
      ~id: string=?,
      ~onKeyDown: React.Event.Keyboard.t => unit=?,
      ~canDismiss: bool=?,
      ~alert: bool=?,
      ~className: string=?,
      ~style: ReactDOM.Style.t=?,
      ~contentRef: React.ref(Js.Nullable.t(Dom.element))=?,
      ~onOpenAutoFocus: focus_event => unit=?,
      ~onCloseAutoFocus: focus_event => unit=?,
      ~children: React.element
    ) =>
    React.element =
    "ModalContent";
};

module Trigger = {
  [@react.component] [@mel.module {js|@radix-ui/react-dialog|js}]
  external make: (~asChild: bool=?, ~children: React.element) => React.element = "Trigger";
};
