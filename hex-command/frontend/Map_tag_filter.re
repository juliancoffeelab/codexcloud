open Ui;
let label = tag =>
  Array.to_list(Js.String.split(~sep={js|-|js}, tag))
  |> List.map(text => {
       let js_text_part_0 = Js.String.toUpperCase(Js.String.slice(~start=0, ~end_=1, text));
       let js_text_part_1 = Js.String.slice(~start=1, text);
       {j|$(js_text_part_0)$(js_text_part_1)|j};
     })
  |> (values => Js.Array.join(~sep={js| |js}, Array.of_list(values)));
[@react.component]
let make = (~options: list((string, int)), ~selected: list(string), ~onChange, ~count) => {
  let (opened, setOpened) = React.useState(() => false);
  let (draft, setDraft) = React.useState(() => selected);
  let toggle = tag =>
    setDraft(tags =>
      List.mem(tag, tags) ? List.filter(value => value != tag, tags) : tags @ [tag]
    );
  let footer =
    <div className={js|map-tag-actions|js}>
      <button onClick={_ => setDraft(_ => [])}> {text({js|Clear|js})} </button>
      <button
        className={js|primary|js}
        onClick={_ => {
          onChange(draft);
          setOpened(_ => false);
        }}>
        {text(
           {
             let js_text_part_1 = string_of_int(count(draft));
             {j|Show $(js_text_part_1) maps|j};
           },
         )}
      </button>
    </div>;
  let trigger =
    <button
      className={
                  let js_text_part_1 = selected == [] ? {js||js} : {js| active|js};
                  {j|map-tag-trigger$(js_text_part_1)|j};
                }
      ariaLabel={
                  let js_text_part_1 =
                    selected == []
                      ? {js|All tags|js}
                      : Js.Array.join(~sep={js|, |js}, Array.of_list(List.map(label, selected)));
                  {j|Filter maps: $(js_text_part_1)|j};
                }
      onClick={_ => {setDraft(_ => selected)}}>
      <svg viewBox={js|0 0 20 20|js} ariaHidden=true>
        <path d={js|M2 5h16M2 10h16M2 15h16M6 3v4M13 8v4M8 13v4|js} />
      </svg>
    </button>;
  <Dialog_window
    trigger
    title={js|Map filters|js}
    opened
    onOpenChange={value => setOpened(_ => value)}
    className={js|map-tag-panel settings-style|js}
    footer>
    <p className={js|help|js}> {text({js|Show maps with any selected tag.|js})} </p>
    {eachList(options, ((tag, total)) =>
       <label key=tag className={js|map-tag-option|js}>
         <input
           type_={js|checkbox|js}
           checked={List.mem(tag, draft)}
           onChange={_ => toggle(tag)}
         />
         <span> {text(label(tag))} </span>
         <small> {text(string_of_int(total))} </small>
       </label>
     )}
  </Dialog_window>;
};
