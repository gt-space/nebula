import { For, createEffect, createSignal } from "solid-js";
import Footer from "../../general-components/Footer";
import { GeneralTitleBar } from "../../general-components/TitleBar";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";
import { appWindow } from "@tauri-apps/api/window";
import { Config, Sequence, State, runSequence, serverIp, StreamState } from "../../comm";

const [configurations, setConfigurations] = createSignal();
const [activeConfig, setActiveConfig] = createSignal();
const [activeBoards, setActiveBoards] = createSignal();

listen('state', (event) => {
  console.log(event.windowLabel);
  setConfigurations((event.payload as State).configs);
  setActiveConfig((event.payload as State).activeConfig);
});

invoke('initialize_state', {window: appWindow});

function AHRS() {
    return <div class="window-template">
    <div style="height: 60px">
      <GeneralTitleBar name="AHRS"/>
    </div>
    <div class="ahrs-view">
      <div class="camera-data-container">
        <div class="ahrs-camera-section">
          <div>Camera</div>
          <button class="button">Camera Enable</button>
          <button class="button">Camera Disable</button>
        </div>

        <div>
          <div>Data</div>
          <div></div>
        </div>
      </div>

      <div class="other-data-container">
        <div class="imu-container">
          <div>IMU</div>

        </div>

        <div class="imu-container">
          <div>Barometer</div>
        </div>

        <div class="imu-container">
          <div>Magnetometer</div>
        </div>
      </div>
    </div>
    
    <div>
      <Footer/>
    </div>
</div>
}

export default AHRS;