import { For, createSignal } from "solid-js";
import { GeneralTitleBar } from "../general-components/TitleBar";
import { Config, Sequence, State, getSequences, runSequence, serverIp, StreamState, stopSequence, sendAbort } from "../comm";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { format } from "date-fns";
import Footer from "../general-components/Footer";

const [configurations, setConfigurations] = createSignal();
const [activeConfig, setActiveConfig] = createSignal();
const [sequences, setSequences] = createSignal();
const [override, setOverride] = createSignal(false);
const [runningSeqs, setRunningSeqs] = createSignal(new Array);

listen('state', (event) => {
  setConfigurations((event.payload as State).configs);
  setActiveConfig((event.payload as State).activeConfig);
  setSequences((event.payload as State).sequences);
});
  
invoke('initialize_state', {window: appWindow});

listen('device_update', (event) => {
  const runningSequences = (event.payload as StreamState).sequences_running;
  setRunningSeqs(runningSequences);
})

function dispatchSequence() {
  try {
    const seqDropdown = document.getElementById("sequenceselect")! as HTMLSelectElement;
    console.log(seqDropdown);
    const timestamp = format(new Date(), 'yyyy-MM-dd_HH-mm-ss');
    const fileName = `${seqDropdown.value}_${timestamp}.py`;
    const currentSequence = (sequences() as Array<Sequence>).find(seq => seq.name === seqDropdown.value);
    if (!currentSequence) {
      throw new Error('Sequence not found');
    }
    const blob = new Blob([currentSequence.script], { type: 'text/plain'});
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    URL.revokeObjectURL(url);
    document.body.removeChild(a);
    console.log(`Sequence saved as ${fileName}`);
    runSequence(serverIp() as string, seqDropdown.value, override());
  } catch (error) {
    console.error('Error dispatching sequence:', error);
  }
}

function Sequnces() {
    return <div class="window-template">
      <div style="height: 60px">
        <GeneralTitleBar name="Sequences"/>
      </div>
      <div class='sequences-view'>
        <div class='sequences-panel'>
          <select
            id="sequenceselect"
            class="sequences-dropdown"
            onChange={(e) => {
              console.log(e?.target.className);
            }}
          >
            <For each={sequences() as Sequence[]}>{(sequence, i) =>
                <option class="seq-dropdown-item" value={sequence.name}>{sequence.name}</option>
              }</For>
          </select>
          <div style={{flex: 1, padding: '5px'}}>
            <button class='toggle-sequence-button' id="sequencebutton" onClick={dispatchSequence}>
              Dispatch Sequence
            </button>
          </div>
        </div>
        <div style={{width: "100%", display: "flex", "justify-content": "center"}}><button class="abort-button" onClick={() => sendAbort(serverIp() as string)}> ABORT </button></div>
        <div style={{"margin-top": "15px", "text-align": "center", width: "100%"}}>Running Sequences:</div>
        <div class="sequences-view-section">
          <For each={runningSeqs() as Array<string>}>{(seq, i) =>
            <div class='running-seq-row'>
              <div>{seq}</div>
              <button class="cancel-seq-button" onClick={() => stopSequence(serverIp() as string, seq)}></button>
            </div>
          }</For>
        </div>
      </div>
      <div>
        <Footer/>
      </div>
  </div>
  }
  
  export default Sequnces;