import { For, createEffect, createSignal } from "solid-js";
import Footer from "../../general-components/Footer";
import { GeneralTitleBar } from "../../general-components/TitleBar";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { Config, Sequence, State, runSequence, serverIp, StreamState, StreamSensor} from "../../comm";
import { emit } from '@tauri-apps/api/event';
import { Valve } from "../../devices";
import SensorSectionView from "../../sensors/SensorSectionView";

const [configurations, setConfigurations] = createSignal();
const [activeConfig, setActiveConfig] = createSignal();
const [label, setLabel] = createSignal('');
const [valves, setValves] = createSignal(new Array<Valve>);
const [sensors, setSensors] =  createSignal(new Array);

const sensorTypes = ['tc', 'pt', 'flow_meter', 'load_cell'];

listen('device_update', (event) => {
  //VALVES UPDATE 
  const valve_object = (event.payload as StreamState).valve_states;
  var valveDevices = Object.keys(valve_object).map((key) => [key, valve_object[key as keyof typeof valve_object]]);
  console.log(valveDevices);
  // updating all valves
  valveDevices.forEach(async (device) => {
    var index = valves().findIndex(item => (item.name === device[0] as string));
    var new_valves = [...valves()];
    console.log(device[1]);
    var valveStates = (device[1] as unknown as object);
    new_valves[index].commanded = valveStates['commanded' as keyof typeof valveStates];
    new_valves[index].actual = valveStates['actual' as keyof typeof valveStates];
    setValves(new_valves);
  });

  //SENSORS UPDATE
  // get sensor data
  const sensor_object = (event.payload as StreamState).sensor_readings;
  var devices = Object.keys(sensor_object).map((key) => [key, sensor_object[key as keyof typeof sensor_object] as StreamSensor]);
  // update data
  console.log(devices);
  devices.forEach((device) => {
    var index = sensors().findIndex(item => (item.name === device[0] as string));
    if (index === -1) {
      return;
    }
    var new_sensors = structuredClone(sensors());
    new_sensors[index].value = (device[1] as StreamSensor).value;
    new_sensors[index].unit = (device[1] as StreamSensor).unit;
    setSensors(new_sensors);
  });
});

listen('state', (event) => {
  console.log("window label " + event.windowLabel);
  setLabel(appWindow.label);
  setConfigurations((event.payload as State).configs);
  setActiveConfig((event.payload as State).activeConfig);
  var activeconfmappings = (configurations() as Config[]).filter((conf) => {return conf.id == activeConfig() as string})[0];

  //valves 
  var vlvs = new Array;
  for (const mapping of activeconfmappings.mappings) {
    if (mapping.sensor_type === 'valve' && mapping.board_id === label().toLowerCase()) {
      vlvs.push(
        {
          name: mapping.text_id,
          group: 'Fuel',
          board_id: mapping.board_id,
          sensor_type: mapping.sensor_type,
          channel: mapping.channel,
          commanded: 'closed',
          actual: 'disconnected'
        } as Valve,
      )
    }
  }
  setValves(vlvs);

  //sensors 
  var snsrs = new Array;
  for (const mapping of activeconfmappings.mappings) {
    if (sensorTypes.includes(mapping.sensor_type) && mapping.board_id === label().toLowerCase()) {
      snsrs.push(
        {
          name: mapping.text_id,
          group: 'Fuel',
          board_id: mapping.board_id,
          sensor_type: mapping.sensor_type,
          channel: mapping.channel,
          unit: '?',
          value: 0,
          offset: NaN,
        });
    }
  }
  setSensors(snsrs);

});

invoke('initialize_state', {window: appWindow});

function stateToColor(state: string) {
  switch (state) {
    case "open":
      return "#22873D";
    case "closed":
      return "#C53434";
    case "disconnected":
      return "#737373";
    case "undetermined":
      return "#015878";
    case "fault":
      return "#CC9A13";
  }
}

function SAM() {
  return <div class="window-template">
    <div style="height: 60px">
      <GeneralTitleBar name={label()}/>
    </div>
    <div class="sam-view">
      <div class="sam-section" id="power">
        <div class="section-title"> POWER</div>
          {/* Power content here */}
      </div>
      <div class="sam-section" id="sensors">
        <div class="section-title"> SENSORS</div>
          <SensorSectionView sensors={sensors()}/>
      </div>
      <div class="sam-section" id="valves">
        <div class="section-title"> VALVES</div>
        <div class="valve-view-section">
          <div style={{display: "grid", "grid-template-columns": "4fr 5fr 10px"}}>
              <div style={{"text-align": "center"}}>Name</div>
              <div style={{display: "flex"}}>
                <div style={{"text-align": "center", flex: 1, "margin-left": "5px"}}>CMD</div>
                <div style={{"text-align": "center", flex: 1}}>ACT</div>
              </div>
              {/* <div style={{"text-align": "center"}}>V</div>
              <div style={{"text-align": "center"}}>C</div> */}
          </div>
          <For each={valves()}>{(valve, i) =>
            <div class='valve-row'>
              <div style="flex: 2; display: flex; justify-content: center;">
                {valves()[i()].name}
              </div>
              <div style="width: 10px; height: 30px; border-left-style:solid; 
                border-left-color: #737373; border-left-width: 1px"></div>
              <div style={{'display': 'flex', 'justify-content': 'center', 'align-items': 'center', 'margin-left': '10px', 'width': '90px', 'height': '10px', 'padding': '5px',"background-color": stateToColor(valves()[i()].commanded)}} >
                {valves()[i()].commanded.charAt(0).toUpperCase()+valves()[i()].commanded.substring(1)}
              </div>
              <div style={{'display': 'flex', 'justify-content': 'center', 'align-items': 'center', 'margin-left': '10px', 'width': '90px', 'height': '10px', 'padding': '5px',"background-color": stateToColor(valves()[i()].actual)}} >
                {valves()[i()].actual.charAt(0).toUpperCase()+valves()[i()].actual.substring(1)}
              </div>
            </div>}
          </For>
        </div>
      </div>
    </div>
    <div>
      <Footer/>
    </div>
</div>
}

export default SAM;