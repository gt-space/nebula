import { Component, createSignal, For, Show } from "solid-js";
import { invoke } from '@tauri-apps/api/tauri'
import { setServerIp, connect, isConnected, setIsConnected, setActivity, serverIp, activity, selfIp, selfPort, sessionId, forwardingId, State, Config, sendActiveConfig, setSessionId, setForwardingId, setSelfIp, setSelfPort, Mapping, sendSequence, Sequence, getConfigs, sendConfig } from "../comm";
import { turnOnLED, turnOffLED } from "../commands";
import { emit, listen } from '@tauri-apps/api/event'
import { appWindow } from "@tauri-apps/api/window";
import { DISCONNECT_ACTIVITY_THRESH } from "../appdata";
import { CodeMirror } from "@solid-codemirror/codemirror";
import { oneDark } from "@codemirror/theme-one-dark";
import { python } from "@codemirror/lang-python";
import { faTrash } from '@fortawesome/free-solid-svg-icons';
import Fa from 'solid-fa';


// states of error message and connect button
const [connectDisplay, setConnectDisplay] = createSignal("Connect");
const [connectionMessage, setConnectionMessage] = createSignal('');
const [showSessionId, setShowSessionId] = createSignal(false);
const [showForwardingId, setShowForwardingId] = createSignal(false);
const [feedsystem, setFeedsystem] = createSignal('Feedsystem_1');
const [activeConfig, setActiveConfig] = createSignal('Config_1');
const [configurations, setConfigurations] = createSignal();
const [currentSequnceText, setCurrentSequenceText] = createSignal('');
const [currentSequnceName, setCurrentSequenceName] = createSignal('');
const [sequences, setSequences] = createSignal();
const [refreshDisplay, setRefreshDisplay] = createSignal("Refresh");
const [saveConfigDisplay, setSaveConfigDisplay] = createSignal("Save");
const [editableEntries, setEditableEntries] = createSignal([{
  text_id: "",
  board_id: 0,
  channel_type: 'GPIO',
  channel: 0,
  computer: 'FLIGHT'
} as Mapping]);
const [configFocusIndex, setConfigFocusIndex] = createSignal(0);
const [subConfigDisplay, setSubConfigDisplay] = createSignal('add');
//configurations()

// function to connect to the server + input validation
async function connectToServer() {
  setConnectDisplay("Connecting...");
  setConnectionMessage('');

  // getting the ip, username, and password from the relevant textfields
  var ip = (document.getElementsByName('server-ip')[0] as HTMLInputElement).value.trim();
  // var username = (document.getElementsByName('username')[0] as HTMLInputElement).value.trim();
  // var password = (document.getElementsByName('password')[0] as HTMLInputElement).value;
  var result = '';

  // presence check on username and password
  // if (username != '' && password != '') {
  //   result = await connect(ip, username, password);
  // } else {
  //   result = 'Please enter a username and password';
  // }

  result = await connect(ip) as string;

  setConnectionMessage(result);
  setConnectDisplay("Connect");
}

// get the activity from the taskbar page
emit('requestActivity');
listen('updateActivity', (event) => {
  setActivity(event.payload as number);
  if (activity() < DISCONNECT_ACTIVITY_THRESH) {
    setIsConnected(true);
  }
});

invoke('initialize_state', {window: appWindow});
listen('state', (event) => {
  setServerIp((event.payload as State).serverIp);
  setIsConnected((event.payload as State).isConnected);
  setSessionId((event.payload as State).sessionId);
  setForwardingId((event.payload as State).forwardingId);
  setSelfIp((event.payload as State).selfIp);
  setSelfPort((event.payload as State).selfPort);
  setConfigurations((event.payload as State).configs);
  setFeedsystem((event.payload as State).feedsystem);
  setActiveConfig((event.payload as State).activeConfig);
  setSequences((event.payload as State).sequences);
  console.log('from listener: ', configurations());
});

// function to close the sessionId info
function closeSessionId(evt:MouseEvent) {
  try {
    var popup = document.getElementById("session-id")!;
    if ((evt.target as HTMLElement).id != popup.id && (evt.target as HTMLElement)!.className != 'id-display'){
      setShowSessionId(false);
    }
  } catch (e) {

  }
}
// function to close the forwardingId info
function closeForwardingId(evt:MouseEvent) {
  try{
    var popup = document.getElementById("forwarding-id")!;
    if ((evt.target as HTMLElement).id != popup.id && (evt.target as HTMLElement)!.className != 'id-display'){
      setShowForwardingId(false);
    }
  } catch (e) {
    
  }
}

document.addEventListener("click", (evt) => closeSessionId(evt));
document.addEventListener("click", (evt) => closeForwardingId(evt));

const Connect: Component = (props) => {
  return <div style="height: 100%; display: flex; flex-direction: column">
    <div style="text-align: center; font-size: 14px">CONNECT</div>
    <div class="system-connect-page">
      <div class="system-connect-section">
        <input class="connect-textfield"
          type="text"
          name="server-ip"
          placeholder="Server IP"
        />
        {/* <input class="connect-textfield"
          type="text"
          name="username"
          placeholder="Username"
        />
        <input class="connect-textfield"
          type="password"
          name="password"
          placeholder="Password"
        /> */}
        <div id="connect-message" style="font-size: 12px">
          {connectionMessage()}
        </div>
        <button class="connect-button" onClick={() => connectToServer()}>
          {connectDisplay()}
        </button>
        <div style="height: 20px"></div>
        <button style="padding: 5px" onClick={() => turnOnLED()}>
          LED test button (on)
        </button>
        <div style="height: 10px"></div>
        <button style="padding: 5px" onClick={() => turnOffLED()}>
          LED test button (off)
        </button>
      </div>
      <div class="system-connect-section">
        <div style="display: grid; grid-template-columns: 1fr 1fr">
          <div style="display: flex; flex-direction: column; margin-right: 20px">
            <div style="text-align: right">Activity:</div>
            <div style="text-align: right">Status:</div>
            <div style="text-align: right">IP:</div>
            <div style="text-align: right">Port:</div>
            <div style="text-align: right">Server IP:</div>
            <div style="text-align: right">Session ID:</div>
            <div style="text-align: right">Forwarding ID:</div>
          </div>
          <div style="display: flex; flex-direction: column; margin-left: 0px">
            <div style="text-align: center" id="activity">{activity()} ms</div>
            <div style="text-align: center" id="status">{isConnected()? "CONNECTED":"DISCONNECTED"}</div>
            <div style="text-align: center">{selfIp() as string}</div>
            <div style="text-align: center">{selfPort() as string}</div>
            <div style="text-align: center">{serverIp() as string}</div>
            <div id="session-id" style="text-align: center">{sessionId() == 'None'? sessionId() as string : 
              <Show 
                when={showSessionId()}
                fallback={<button id="session-id" class="connect-info-button" 
                onClick={() => {setShowSessionId(true); console.log(showSessionId())}}>Click to view</button>}
              >
                <div class='id-display'>{sessionId() as string}</div>
              </Show>}
            </div>
            <div style="text-align: center" id="forwarding-id">{forwardingId() == 'None'? forwardingId() as string : 
              <Show 
                when={showForwardingId()}
                fallback={<button id="forwarding-id" class="connect-info-button" 
                onClick={() => {setShowForwardingId(true); console.log(showForwardingId())}}>Click to view</button>}
              >
                <div class='id-display'>{forwardingId() as string}</div>
              </Show>}
            </div>
          </div>
        </div>
      </div>
    </div>
</div>
}

async function setFeedsystemAndActiveConfig() {
  var feedsystem = (document.querySelector('input[name="feedsystem-select"]:checked')! as HTMLInputElement);
  console.log(feedsystem.value);
  var dropdown = (document.getElementById("feed-config-drop-1"))! as HTMLSelectElement;
  console.log(dropdown.value);
  await invoke('update_feedsystem', {window: appWindow, value: feedsystem.value});
  await invoke('update_active_config', {window: appWindow, value: dropdown.value});
  sendActiveConfig(serverIp() as string, dropdown.value);
  setActiveConfig(dropdown.value);

}
//get state updates
// invoke('initialize_state', {window: appWindow});
// listen('state', (event) => {
//   console.log((event.payload as State).feedsystem);
//   setFeedsystem((event.payload as State).feedsystem);
//   setActiveConfig((event.payload as State).activeConfig);
  
// });

async function setFeedsystemData() {
  await new Promise(r => setTimeout(r, 100));
  var feedsystemToSet = document.querySelectorAll('input[value="'+(feedsystem() as string)+'"]')[0];
  var dropdown = (document.getElementById("feed-config-drop-1"))! as HTMLSelectElement;
  console.log(feedsystemToSet);
  (feedsystemToSet as HTMLInputElement)!.checked = true
  console.log(activeConfig());
  console.log(dropdown);
  dropdown.value = activeConfig();
}  

async function refreshConfigs() {
  setRefreshDisplay("Refreshing...");
  var ip = serverIp() as string;
  await getConfigs(ip);
  var configs = await getConfigs(ip);
  console.log(configs);
  if (configs instanceof Error) {
    setRefreshDisplay('Error!');
    await new Promise(r => setTimeout(r, 1000));
    setRefreshDisplay('Refresh');
    return;
  }
  var configMap = new Map(Object.entries(configs));
  var configArray = Array.from(configMap, ([name, value]) => ({'id': name, 'mappings': value }));
  await invoke('update_configs', {window: appWindow, value: configArray});
  setRefreshDisplay('Refreshed!');
  await new Promise(r => setTimeout(r, 1000));
  setRefreshDisplay('Refresh');
  console.log(configurations());
}

const Feedsystem: Component = (props) => {
  listen('state', (event) => {
    setFeedsystem((event.payload as State).feedsystem);
    setActiveConfig((event.payload as State).activeConfig);
  });
  setFeedsystemData();
  return <div style="height: 100%; display: flex; flex-direction: column">
    <div style="text-align: center; font-size: 14px">FEEDSYSTEM</div>
    <div class='select-feedsystem-body'>
      <div style={{'display': 'flex', 'flex-direction': 'row'}}>
      <div style={{'width': '200px','padding': '20px'}}> 
        <div style={{"margin-bottom": '10px'}}>Select feedsystem:</div>
        <div style={{'margin-left': '20px', 'display': 'flex', "flex-direction": 'column', 'align-items': 'flex-start'}}>
          <div style={{'display': 'flex', "flex-direction": 'row', "align-items": "center", 'padding-top': '5px'}}>
              <input class='radiobutton' style={{'margin': '10px'}} type="radio" name="feedsystem-select" value="Feedsystem_1" id="Feedystem-id-1" checked></input>
              <div>
                Feedsystem 1
              </div>
          </div>
          <div style={{'display': 'flex', "flex-direction": 'row', "align-items": "center", 'padding-top': '5px'}}>
              <input class='radiobutton' style={{'margin': '10px'}} type="radio" name="feedsystem-select" value="Feedsystem_2" id="Feedystem-id-2"></input>
              <div>
                Feedsystem 2
              </div>
          </div>
          <div style={{'display': 'flex', "flex-direction": 'row', "align-items": "center", 'padding-top': '5px'}}>
              <input class='radiobutton' style={{'margin': '10px'}} type="radio" name="feedsystem-select" value="Feedsystem_3" id="Feedystem-id-3"></input>
              <div>
                Feedsystem 3
              </div>
          </div>
          <div style={{'display': 'flex', "flex-direction": 'row', "align-items": "center", 'padding-top': '5px'}}>
              <input class='radiobutton' style={{'margin': '10px'}} type="radio" name="feedsystem-select" value="Feedsystem_4" id="Feedystem-id-4"></input>
              <div>
                Feedsystem 4
              </div>
          </div>
        </div>
      </div>
      <div style={{'flex': 2, 'padding-top': '20px'}}>
        <div style={{"margin-bottom": '10px'}}>Set configuration:</div>
        <div style={{'display': 'flex', "flex-direction": 'column', 'align-items': 'flex-start'}}>
          <div>
            <select
              id="feed-config-drop-1"
              class="feedsystem-config-dropdown"
              onChange={(e) => {
                console.log(e?.target.className);
              }}
            >
            <For each={configurations() as Config[]}>{(config, i) =>
              <option class="conf-dropdown-item" value={config.id}>{config.id}</option>
            }</For>
            {/* <option class="seq-dropdown-item" value="seq1">Config 1</option>
            <option class="seq-dropdown-item" value="seq2">Config 2</option>
            <option class="seq-dropdown-item" value="seq3">Config 3</option>
            <option class="seq-dropdown-item" value="seq4">Config 4</option>
            <option class="seq-dropdown-item" value="seq5">Config 5</option>
            <option class="seq-dropdown-item" value="seq6">Config 6</option> */}
          </select>
          </div>
          <button style={{"margin": '5px'}} class='refresh-button' onClick={refreshConfigs}>{refreshDisplay()}</button>        
        </div>
      </div>
      </div>
      <div style={{'margin-left': '10px', 'margin-top': '10px','padding-left': '170px'}}>
        <button class='submit-feedsystem-button' onClick={setFeedsystemAndActiveConfig}> Submit </button>
      </div>
    </div>
</div>
}

function addNewConfigEntry() {
  var entries = [...editableEntries()];
  entries.push({
    text_id: "",
    board_id: 0,
    channel_type: 'gpio',
    channel: 0,
    computer: 'FLIGHT'
  } as Mapping);
  setEditableEntries(entries);
  console.log(editableEntries());
}

function deleteConfigEntry(entry: Mapping) {
  if (editableEntries().length === 1) {
    setEditableEntries([{
        text_id: "",
        board_id: 0,
        channel_type: 'gpio',
        channel: 0,
        computer: 'FLIGHT'
      } as Mapping]);
      return;
  }
  var entries = [...editableEntries()];
  var mappingnames = document.querySelectorAll("[id=addmappingname]") as unknown as Array<HTMLInputElement>;
  var mappingboardids = document.querySelectorAll("[id=addmappingboardid]") as unknown as Array<HTMLInputElement>;
  var mappingchanneltypes = document.querySelectorAll("[id=addmappingchanneltype]") as unknown as Array<HTMLSelectElement>;
  var mappingchannels = document.querySelectorAll("[id=addmappingchannel]") as unknown as Array<HTMLInputElement>;
  var mappingcomputers = document.querySelectorAll("[id=addmappingcomputer]") as unknown as Array<HTMLSelectElement>;
  for (var i = 0; i < entries.length; i++) {
    entries[i].text_id = mappingnames[i].value;
    entries[i].board_id = mappingboardids[i].value as unknown as number;
    entries[i].channel_type = mappingchanneltypes[i].value;
    entries[i].channel = mappingchannels[i].value as unknown as number;
    entries[i].computer = mappingcomputers[i].value
  }
  console.log(entry);
  entries.splice(entries.indexOf(entry), 1);
  setEditableEntries(entries);
  console.log('deleted somthing!');
  console.log(editableEntries());
}

async function submitConfig(edited: boolean) {
  var newConfigNameInput = (document.getElementById('newconfigname') as HTMLInputElement)!;
  var configName;
  if (edited) {
    configName = (configurations() as Config[])[configFocusIndex()].id;
  } else {
    configName = newConfigNameInput.value;
    if (configName === "") {
      setSaveConfigDisplay("Error!");
      newConfigNameInput.value = 'Enter a name here!';
      await new Promise(r => setTimeout(r, 1000));
      setSaveConfigDisplay("Save");
      newConfigNameInput.value = '';
      return;
    }
  }
  setSaveConfigDisplay("Saving...");
  var entries = [...editableEntries()];
  var mappingnames = document.querySelectorAll("[id=addmappingname]") as unknown as Array<HTMLInputElement>;
  var mappingboardids = document.querySelectorAll("[id=addmappingboardid]") as unknown as Array<HTMLInputElement>;
  var mappingchanneltypes = document.querySelectorAll("[id=addmappingchanneltype]") as unknown as Array<HTMLSelectElement>;
  var mappingchannels = document.querySelectorAll("[id=addmappingchannel]") as unknown as Array<HTMLInputElement>;
  var mappingcomputers = document.querySelectorAll("[id=addmappingcomputer]") as unknown as Array<HTMLSelectElement>;
  for (var i = 0; i < entries.length; i++) {
    entries[i].text_id = mappingnames[i].value;
    entries[i].board_id = mappingboardids[i].value as unknown as number;
    entries[i].channel_type = mappingchanneltypes[i].value.replace(' ', '_').toLowerCase();
    entries[i].channel = mappingchannels[i].value as unknown as number;
    entries[i].computer = mappingcomputers[i].value.toLowerCase();
  }
  console.log(entries);
  var success = await sendConfig(serverIp() as string, {id: configName, mappings: entries} as Config);
  console.log(success);
  if (success instanceof Error && !(success instanceof SyntaxError)) {
    setSaveConfigDisplay("Error!");
    await new Promise(r => setTimeout(r, 1000));
    setSaveConfigDisplay("Save");
    return;
  }
  setSaveConfigDisplay("Saved!");
  refreshConfigs();
  await new Promise(r => setTimeout(r, 1000));
  setSaveConfigDisplay("Save");
}

const AddConfigView: Component = (props) => {
  return <div style={{width: '100%'}}>
    <div class="add-config-section">
      <div class="add-config-setup">
        <p>Add new config:</p>
        <input id='newconfigname' class="add-config-input" type="text" placeholder="Name"/>
      </div>
      <div class="add-config-btns">
        <button class="add-config-btn" onClick={addNewConfigEntry}>Insert Mapping</button>
        <button style={{"background-color": '#C53434'}} class="add-config-btn" onClick={function(event){
          setEditableEntries([{
            text_id: "",
            board_id: 0,
            channel_type: 'gpio',
            channel: 0,
            computer: 'FLIGHT'
          } as Mapping]);
        }}>Cancel</button>
        <button style={{"background-color": '#015878'}} class="add-config-btn" onClick={() => {submitConfig(false);}}>{saveConfigDisplay()}</button>
      </div>
    </div>
    <div class="horizontal-line"></div>
    <div style={{"max-height": '20%', "overflow-y": "auto"}}>
      <For each={editableEntries()}>{(entry, i) =>
          <div class="add-config-configurations">
            <input id={"addmappingname"} type="text" value={entry.text_id} placeholder="Name" class="add-config-styling"/>
            <input type="text" name="" id={"addmappingboardid"} value={entry.board_id} placeholder="Board ID" class="add-config-styling"/>
            <select name="" id={"addmappingchanneltype"} value={entry.channel_type.toUpperCase()} class="add-config-styling">
              <option class="seq-dropdown-item">GPIO</option>
              <option class="seq-dropdown-item">LED</option>
              <option class="seq-dropdown-item">RAIL 3V3</option>
              <option class="seq-dropdown-item">RAIL 5V</option>
              <option class="seq-dropdown-item">RAIL 5V5</option>
              <option class="seq-dropdown-item">RAIL 24V</option>
              <option class="seq-dropdown-item">CURRENT LOOP</option>
              <option class="seq-dropdown-item">DIFFERENTIAL SIGNAL</option>
              <option class="seq-dropdown-item">TC</option>
              <option class="seq-dropdown-item">RTD</option>
              <option class="seq-dropdown-item">VALVE</option>
              <option class="seq-dropdown-item">VALVE CURRENT</option>
              <option class="seq-dropdown-item">VALVE VOLTAGE</option>
            </select>
            <input type="text" name="" id={"addmappingchannel"} value={entry.channel} placeholder="Channel" class="add-config-styling"/>
            <select name="" id={"addmappingcomputer"} value={entry.computer as string} class="add-config-styling">
              <option class="seq-dropdown-item">FLIGHT</option>
              <option class="seq-dropdown-item">GROUND</option>
            </select>
            <div onClick={() => deleteConfigEntry(entry)}><Fa icon={faTrash} color='#C53434'/></div>
          </div>
        }
      </For>
    </div>
  </div>
}

function loadConfigEntries(index: number) {
  var entries: Mapping[] = [];
  (configurations() as Config[])[index].mappings.forEach( (value) => {
    entries.push({
      text_id: value.text_id,
      board_id: value.board_id,
      channel_type: value.channel_type.replace('_', ' ').toUpperCase(),
      channel: value.channel,
      computer: value.computer.toUpperCase()
    });
  });
  setEditableEntries(entries);
}

const EditConfigView: Component<{index: number}> = (props) => {
  var index = props.index;
  loadConfigEntries(index);
  return <div style={{width: '100%'}}>
    <div class="add-config-section">
      <div class="add-config-setup">
        <p>Editing config:</p>
        <div style={{"font-weight": "bold"}}>{(configurations() as Config[])[index].id}</div>
      </div>
      <div class="add-config-btns">
        <button class="add-config-btn" onClick={addNewConfigEntry}>Insert Mapping</button>
        <button style={{"background-color": '#C53434'}} class="add-config-btn" onClick={function(event){
          setEditableEntries([{
            text_id: "",
            board_id: 0,
            channel_type: 'gpio',
            channel: 0,
            computer: 'FLIGHT'
          } as Mapping]);
          setSubConfigDisplay('view');
        }}>Cancel</button>
        <button style={{"background-color": '#015878'}} class="add-config-btn" onClick={async () => {await submitConfig(true); setSubConfigDisplay('view');}}>{saveConfigDisplay()}</button>
      </div>
    </div>
    <div class="horizontal-line"></div>
    <div style={{"max-height": '20%', "overflow-y": "auto"}}>
      <For each={editableEntries()}>{(entry, i) =>
          <div class="add-config-configurations">
            <input id={"addmappingname"} type="text" value={entry.text_id} placeholder="Name" class="add-config-styling"/>
            <input type="text" name="" id={"addmappingboardid"} value={entry.board_id} placeholder="Board ID" class="add-config-styling"/>
            <select name="" id={"addmappingchanneltype"} value={entry.channel_type.toUpperCase()} class="add-config-styling">
              <option class="seq-dropdown-item">GPIO</option>
              <option class="seq-dropdown-item">LED</option>
              <option class="seq-dropdown-item">RAIL 3V3</option>
              <option class="seq-dropdown-item">RAIL 5V</option>
              <option class="seq-dropdown-item">RAIL 5V5</option>
              <option class="seq-dropdown-item">RAIL 24V</option>
              <option class="seq-dropdown-item">CURRENT LOOP</option>
              <option class="seq-dropdown-item">DIFFERENTIAL SIGNAL</option>
              <option class="seq-dropdown-item">TC</option>
              <option class="seq-dropdown-item">RTD</option>
              <option class="seq-dropdown-item">VALVE</option>
              <option class="seq-dropdown-item">VALVE CURRENT</option>
              <option class="seq-dropdown-item">VALVE VOLTAGE</option>
            </select>
            <input type="text" name="" id={"addmappingchannel"} value={entry.channel} placeholder="Channel" class="add-config-styling"/>
            <select name="" id={"addmappingcomputer"} value={entry.computer as string} class="add-config-styling">
              <option class="seq-dropdown-item">FLIGHT</option>
              <option class="seq-dropdown-item">GROUND</option>
            </select>
            <div onClick={() => deleteConfigEntry(entry)}><Fa icon={faTrash} color='#C53434'/></div>
          </div>
        }
      </For>
    </div>
  </div>
}

const DisplayConfigView: Component<{index: number}> = (props) => {
  var index = props.index;
  return <div style={{width: '100%'}}>
    <div class="add-config-section">
      <div class="add-config-setup">
        <p>Viewing config:</p>
        <div style={{"font-weight": "bold"}}>{(configurations() as Config[])[index].id}</div>
      </div>
      <div class="add-config-btns">
      <button class="add-config-btn" onClick={()=>{setSubConfigDisplay('edit')}}>Edit</button>
      <button class="add-config-btn" onClick={()=>{setSubConfigDisplay('add');}}>Exit</button>
      </div>
    </div>
    <div class="horizontal-line"></div>
    <div style={{"margin-top": '5px'}} class="add-config-configurations">
      <div style={{width: '20%', "text-align": 'center'}}>Name</div>
      <div style={{width: '20%', "text-align": 'center'}}>Board ID</div>
      <div style={{width: '20%', "text-align": 'center'}}> Channel Type</div>
      <div style={{width: '20%', "text-align": 'center'}}>Channel</div>
      <div style={{width: '20%', "text-align": 'center'}}>Computer</div>
    </div>
    <div style={{"max-height": '20%', "overflow-y": "auto"}}>
      <For each={(configurations() as Config[])[index].mappings}>{(entry, i) =>
        <div class="add-config-configurations">
          <div style={{width: '20%', "text-align": 'center', "font-family": 'RubikLight'}}>{entry.text_id}</div>
          <div style={{width: '20%', "text-align": 'center', "font-family": 'RubikLight'}}>{entry.board_id}</div>
          <div style={{width: '20%', "text-align": 'center', "font-family": 'RubikLight'}}>{entry.channel_type.replace('_', ' ').toUpperCase()}</div>
          <div style={{width: '20%', "text-align": 'center', "font-family": 'RubikLight'}}>{entry.channel}</div>
          <div style={{width: '20%', "text-align": 'center', "font-family": 'RubikLight'}}>{entry.computer.toUpperCase()}</div>
        </div>
        }
      </For>
    </div>
  </div>
}

const ConfigView: Component = (props) => {
  setEditableEntries([{
    text_id: "",
    board_id: 0,
    channel_type: 'gpio',
    channel: 0,
    computer: 'FLIGHT'
  } as Mapping]);
  return <div style="height: 100%">
    <div style="text-align: center; font-size: 14px">CONFIGURATION</div>
    <div class="system-config-page">
      <div class="system-connect-section">
        <div style={{display: "grid", "grid-template-columns": "100px 1fr 100px", width: '100%', "margin-bottom": '5px'}}>
          <div></div>
          <div style="text-align: center; font-size: 14px; font-family: 'Rubik'">Available Configurations</div>
          <button style={{"justify-content": "end"}} class="refresh-button" onClick={refreshConfigs}>{refreshDisplay()}</button>
        </div>
        
        <div class="horizontal-line"></div>
        <div class="existing-configs-sections">
          <div style={{height: '5px'}}></div>
          <div style={{display: "flex", "justify-content": "space-between"}}>
            <div style={{"padding-left": '20px'}} class="existing-config-row-subheadings">Name</div>
            <div style={{"padding-right": '20px'}} class="existing-config-row-subheadings">Date</div>
          </div>
          <div style={{height: "5px"}}></div>
          <div style={{"overflow-y": "auto", "max-height": '100px'}}>
            <For each={configurations() as Config[]}>{(config, i) =>
                <div class="existing-config-row" onClick={()=>{if (subConfigDisplay() != 'view') {setSubConfigDisplay('view'); setConfigFocusIndex(i as unknown as number);}}}>
                  {config.id}
                </div>
              }
            </For>
          </div>
        </div>
      </div>
      <div class="system-connect-section">
        {(() => {
          console.log('some display set');
          console.log(configFocusIndex());
          if (subConfigDisplay() == 'add') {
            return <AddConfigView />;
          } else if (subConfigDisplay() == 'view') {
            return <DisplayConfigView index={configFocusIndex()} />;
          } else if (subConfigDisplay() == 'edit') {
            return <EditConfigView index={configFocusIndex()} />;
          } else {
            return <div>How did we get here??</div>;
          }
        })()}
      </div>
    </div>
</div>
}

const Sequences: Component = (props) => {
  return <div style="height: 100%">
    <div style="text-align: center; font-size: 14px">SEQUENCES</div>
    <div class="system-sequences-page">
      <div class="sequences-list-view">
        <div style={{display: "grid", "grid-template-columns": "100px 1fr 100px", width: '100%', "margin-bottom": '5px'}}>
          <div></div>
          <div style="text-align: center; font-size: 14px; font-family: 'Rubik'">Available Sequences</div>
          <button style={{"justify-content": "end"}} class="refresh-button" onClick={()=>{}}>{refreshDisplay()}</button>
        </div>
        
        <div class="horizontal-line"></div>
        <div>
          <For each={sequences() as Sequence[]}>{(seq, i) =>
              <div class="sequence-display-item">
                {seq.name}
              </div>
            }
          </For>
        </div>
      </div>
      <div class="sequences-editor">
        <div style={{display: "grid", "grid-template-columns": "300px 1fr", height: '50px'}}>
          <input class="connect-textfield"
            type="text"
            name="sequence-name"
            placeholder="Sequence Name"
            value={currentSequnceName()}
            onInput={(event) => setCurrentSequenceName(event.currentTarget.value)}
          style={{width: '200px'}}/>
          <div style={{width: '100%'}}><button style={{float: "right"}} class="submit-feedsystem-button" onClick={() => sendSequence(serverIp() as string, currentSequnceName(), btoa(currentSequnceText()))}> Submit Sequence </button></div>
        </div>
        <div class="code-editor">
          <CodeMirror style={{height: "100%"}} value={currentSequnceText()} onValueChange={(value) => {setCurrentSequenceText(value);}} extensions={[python()]} theme={oneDark}/>
        </div>
      </div>
    </div>
</div>
}

export {Connect, Feedsystem, ConfigView, Sequences};