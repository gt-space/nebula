/* @refresh reload */
import { render } from "solid-js/web";

import "../style.css";
import Pid from "./Pid";

render(() => <Pid/>, document.getElementById("root") as HTMLElement);