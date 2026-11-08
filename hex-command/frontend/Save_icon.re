[@react.component]
let make = (~name) => {
  let path =
    switch (name) {
    | "load" => "M5 4v16l14-8L5 4z"
    | "gold" => "M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12"
    | "villages" => "m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7"
    | "objectives" => "M5 21V3m0 0h14l-4 5 4 5H5"
    | "units" => "M5 21v-3a7 7 0 0 1 14 0v3"
    | "trash" => "M3 7h18M9 7V3h6v4M5 7l1 14h12l1-14M10 11v6M14 11v6"
    | _ => "m15 15 6 6"
    };
  <svg
    className={"save-icon" ++ (name == "load" ? " filled" : "")}
    viewBox="0 0 24 24"
    ariaHidden=true
    focusable="false">
    {switch (name) {
     | "details" => <circle cx="10" cy="10" r="6" />
     | "gold" => <circle cx="12" cy="12" r="9" />
     | "units" => <circle cx="12" cy="6.5" r="3.5" />
     | _ => React.null
     }}
    <path d=path />
  </svg>;
};
