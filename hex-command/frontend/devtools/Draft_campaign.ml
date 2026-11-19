type profile =
  | Coast
  | Island_city
  | River_crossing
  | Floodplain
  | Woodland
  | Ridge_pass
  | Badlands
  | Caverns
  | Cave_frontier
  | Rift_caverns

type intent = Transit | Evacuation | Defense | Siege | Exploration

let families =
  [
    ("coast", Coast, [ Exploration; Siege ]);
    ("island-city", Island_city, [ Siege; Transit ]);
    ("river-crossing", River_crossing, [ Transit; Evacuation; Defense ]);
    ("floodplain", Floodplain, [ Exploration; Transit; Evacuation ]);
    ("woodland", Woodland, [ Defense; Siege; Exploration ]);
    ("ridge-pass", Ridge_pass, [ Transit; Evacuation; Exploration ]);
    ("badlands", Badlands, [ Transit; Evacuation; Exploration ]);
    ("caverns", Caverns, [ Exploration; Transit; Siege ]);
    ("cave-frontier", Cave_frontier, [ Transit; Exploration ]);
    ("rift-caverns", Rift_caverns, [ Exploration; Transit ]);
  ]

let intent_name = function
  | Transit -> "transit"
  | Evacuation -> "evacuation"
  | Defense -> "defense"
  | Siege -> "siege"
  | Exploration -> "exploration"

let choices intents = String.concat ", " (List.map intent_name intents)

let help =
  String.concat "\n"
    (List.map (fun (name, _, intents) -> "         " ^ name ^ ": " ^ choices intents) families)

let find name = List.find_opt (fun (key, _, _) -> key = name) families

let intent name intents = function
  | None -> List.hd intents
  | Some value -> (
      match List.find_opt (fun intent -> intent_name intent = value) intents with
      | Some intent -> intent
      | None ->
          let options = choices intents in
          invalid_arg {j|Supported intents for $(name): $(options)|j})

let facing = function
  | None | Some "east" -> Draft_layout.East
  | Some "west" -> Draft_layout.West
  | Some _ -> invalid_arg {js|--facing requires east or west|js}

let create (name, profile, intents) ~intent:requested_intent ~facing:requested_facing ~seed ~cols
    ~rows ~tiles ~editable ~ids =
  let intent = intent name intents requested_intent and facing = facing requested_facing in
  if cols < 24 || rows < 20 then
    invalid_arg {js|Campaign profiles require at least 24 columns and 20 rows.|js};
  let canvas = Draft_layout.create ~seed ~cols ~rows ~facing ~tiles ~editable ~ids in
  let near_edge = intent = Evacuation
  and exploration = intent = Exploration
  and siege = intent = Siege in
  (match profile with
  | Coast -> Draft_water.coast canvas ~siege ~more_islands:exploration
  | Island_city | River_crossing | Floodplain ->
      Draft_water.river canvas ~island:(profile = Island_city) ~flood:(profile = Floodplain)
        ~near_edge
        ~additional:(siege || exploration || intent = Defense)
        ~defense:(intent = Defense)
  | Woodland -> Draft_surface.woodland canvas ~siege ~exploration
  | Ridge_pass -> Draft_surface.ridge canvas ~near_edge ~alternate:exploration
  | Badlands -> Draft_surface.badlands canvas ~near_edge ~exploration
  | Caverns | Cave_frontier | Rift_caverns ->
      Draft_caverns.create canvas ~frontier:(profile = Cave_frontier) ~rift:(profile = Rift_caverns)
        ~exploration ~siege);
  canvas
