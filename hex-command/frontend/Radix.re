module AlertCancel = {
  [@react.component] [@mel.module {js|@radix-ui/react-alert-dialog|js}]
  external make: (~asChild: bool=?, ~children: React.element) => React.element = "Cancel";
};
module Radio = {
  module Root = {
    [@react.component] [@mel.module {js|@radix-ui/react-radio-group|js}]
    external make:
      (
        ~asChild: bool=?,
        ~value: string,
        ~onValueChange: string => unit,
        ~children: React.element
      ) =>
      React.element =
      "Root";
  };
  module Item = {
    [@react.component] [@mel.module {js|@radix-ui/react-radio-group|js}]
    external make:
      (~asChild: bool=?, ~value: string, ~disabled: bool=?, ~children: React.element) =>
      React.element =
      "Item";
  };
};
module Tabs = {
  module Root = {
    [@react.component] [@mel.module {js|@radix-ui/react-tabs|js}]
    external make:
      (
        ~asChild: bool=?,
        ~value: string,
        ~onValueChange: string => unit,
        ~children: React.element
      ) =>
      React.element =
      "Root";
  };
  module List = {
    [@react.component] [@mel.module {js|@radix-ui/react-tabs|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "List";
  };
  module Trigger = {
    [@react.component] [@mel.module {js|@radix-ui/react-tabs|js}]
    external make:
      (~asChild: bool=?, ~value: string, ~disabled: bool=?, ~children: React.element) =>
      React.element =
      "Trigger";
  };
  module Content = {
    [@react.component] [@mel.module {js|@radix-ui/react-tabs|js}]
    external make: (~asChild: bool=?, ~value: string, ~children: React.element) => React.element =
      "Content";
  };
};
module Collapsible = {
  module Root = {
    [@react.component] [@mel.module {js|@radix-ui/react-collapsible|js}]
    external make:
      (
        ~asChild: bool=?,
        ~_open: bool=?,
        ~onOpenChange: bool => unit=?,
        ~children: React.element
      ) =>
      React.element =
      "Root";
  };
  module Trigger = {
    [@react.component] [@mel.module {js|@radix-ui/react-collapsible|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "Trigger";
  };
  module Content = {
    [@react.component] [@mel.module {js|@radix-ui/react-collapsible|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "Content";
  };
};
module Accordion = {
  module Root = {
    [@react.component] [@mel.module {js|@radix-ui/react-accordion|js}]
    external make:
      (~asChild: bool=?, ~_type: string, ~collapsible: bool=?, ~children: React.element) =>
      React.element =
      "Root";
  };
  module Item = {
    [@react.component] [@mel.module {js|@radix-ui/react-accordion|js}]
    external make: (~asChild: bool=?, ~value: string, ~children: React.element) => React.element =
      "Item";
  };
  module Header = {
    [@react.component] [@mel.module {js|@radix-ui/react-accordion|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "Header";
  };
  module Trigger = {
    [@react.component] [@mel.module {js|@radix-ui/react-accordion|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "Trigger";
  };
  module Content = {
    [@react.component] [@mel.module {js|@radix-ui/react-accordion|js}]
    external make: (~asChild: bool=?, ~children: React.element) => React.element = "Content";
  };
};
