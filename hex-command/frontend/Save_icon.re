[@react.component]
let make = (~name) => {
  let path =
    switch (name) {
    | matched_text_0 when matched_text_0 == {js|load|js} => {js|M5 4v16l14-8L5 4z|js}
    | matched_text_1 when matched_text_1 == {js|gold|js} => {js|M14 8h-3a2 2 0 0 0 0 4h2a2 2 0 0 1 0 4h-3m2-10v12|js}
    | matched_text_2 when matched_text_2 == {js|villages|js} => {js|m3 11 9-8 9 8M5 10v11h14V10M10 21v-7h4v7|js}
    | matched_text_3 when matched_text_3 == {js|objectives|js} => {js|M5 21V3m0 0h14l-4 5 4 5H5|js}
    | matched_text_4 when matched_text_4 == {js|units|js} => {js|M5 21v-3a7 7 0 0 1 14 0v3|js}
    | matched_text_5 when matched_text_5 == {js|trash|js} => {js|M3 7h18M9 7V3h6v4M5 7l1 14h12l1-14M10 11v6M14 11v6|js}
    | _ => {js|m15 15 6 6|js}
    };
  <svg
    className={
                let js_text_part_1 = name == {js|load|js} ? {js| filled|js} : {js||js};
                {j|save-icon$(js_text_part_1)|j};
              }
    viewBox={js|0 0 24 24|js}
    ariaHidden=true
    focusable={js|false|js}>
    {switch (name) {
     | matched_text_6 when matched_text_6 == {js|details|js} =>
       <circle cx={js|10|js} cy={js|10|js} r={js|6|js} />
     | matched_text_7 when matched_text_7 == {js|gold|js} =>
       <circle cx={js|12|js} cy={js|12|js} r={js|9|js} />
     | matched_text_8 when matched_text_8 == {js|units|js} =>
       <circle cx={js|12|js} cy={js|6.5|js} r={js|3.5|js} />
     | _ => React.null
     }}
    <path d=path />
  </svg>;
};
