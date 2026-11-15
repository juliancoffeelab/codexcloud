open Ui;

let label = tag =>
  String.split_on_char('-', tag) |> List.map(String.capitalize_ascii) |> String.concat(" ");

[@react.component]
let make = (~options: list((string, int)), ~selected: list(string), ~onChange, ~count) => {
  let dialog = React.useRef(Js.Nullable.null);
  let (draft, setDraft) = React.useState(() => selected);
  let toggle = tag =>
    setDraft(tags =>
      List.mem(tag, tags) ? List.filter(value => value != tag, tags) : tags @ [tag]
    );
  let footer =
    <div className="map-tag-actions">
      <button onClick={_ => setDraft(_ => [])}> {text("Clear")} </button>
      <button
        className="primary"
        onClick={_ => {
          onChange(draft);
          close(dialog);
        }}>
        {text("Show " ++ string_of_int(count(draft)) ++ " maps")}
      </button>
    </div>;
  <>
    <button
      className={"map-tag-trigger" ++ (selected == [] ? "" : " active")}
      ariaLabel={Utf8.to_js(
        "Filter maps: "
        ++ (selected == [] ? "All tags" : String.concat(", ", List.map(label, selected))),
      )}
      ariaHaspopup="dialog"
      onClick={_ => {
        setDraft(_ => selected);
        show(dialog);
      }}>
      <svg viewBox="0 0 20 20" ariaHidden=true>
        <path d="M2 5h16M2 10h16M2 15h16M6 3v4M13 8v4M8 13v4" />
      </svg>
    </button>
    <Dialog_window title="Map filters" dialog className="map-tag-panel settings-style" footer>
      <p className="help"> {text("Show maps with any selected tag.")} </p>
      {eachList(options, ((tag, total)) =>
         <label key=tag className="map-tag-option">
           <input type_="checkbox" checked={List.mem(tag, draft)} onChange={_ => toggle(tag)} />
           <span> {text(label(tag))} </span>
           <small> {text(string_of_int(total))} </small>
         </label>
       )}
    </Dialog_window>
  </>;
};
