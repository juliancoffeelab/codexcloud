(* Parse attributes with the pinned compiler; comments and string contents are not imports. *)
let iterator =
  {
    Ast_iterator.default_iterator with
    attribute =
      (fun self attribute ->
        let name = attribute.Parsetree.attr_name in
        if name.txt = "mel.module" || name.txt = "bs.module" then
          Location.raise_errorf ~loc:name.loc
            "Foreign module imports are forbidden; use package-owned OCaml data";
        Ast_iterator.default_iterator.attribute self attribute);
  }

let parse file buffer =
  Location.init buffer file;
  if Filename.check_suffix file ".mli" || Filename.check_suffix file ".rei" then
    iterator.signature iterator (Parse.interface buffer)
  else iterator.structure iterator (Parse.implementation buffer)

let refmt = ref "refmt"

let check file =
  if Filename.check_suffix file ".re" || Filename.check_suffix file ".rei" then
    let interface = string_of_bool (Filename.check_suffix file ".rei") in
    let channel =
      Unix.open_process_args_in !refmt
        [| !refmt; "--parse"; "re"; "--print"; "ml"; "--interface"; interface; file |]
    in
    let source =
      Fun.protect
        ~finally:(fun () ->
          match Unix.close_process_in channel with
          | Unix.WEXITED 0 -> ()
          | _ -> failwith ("Reason parsing failed: " ^ file))
        (fun () -> In_channel.input_all channel)
    in
    parse file (Lexing.from_string source)
  else
    let channel = open_in file in
    Fun.protect
      ~finally:(fun () -> close_in channel)
      (fun () -> parse file (Lexing.from_channel channel))

let () =
  let files = ref [] and failed = ref false in
  Arg.parse
    [ ("--refmt", Arg.Set_string refmt, "Pinned Reason parser") ]
    (fun file -> files := file :: !files)
    "migration_import_check [--refmt path] sources...";
  List.iter
    (fun file ->
      try check file
      with error ->
        failed := true;
        Location.report_exception Format.err_formatter error)
    (List.rev !files);
  if !failed then exit 1
