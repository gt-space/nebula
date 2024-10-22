import * as go from 'gojs';
import { onMount, onCleanup } from 'solid-js';
import { GeneralTitleBar } from "../general-components/TitleBar";
import Footer from "../general-components/Footer";

import PR_FIL from '../assets/P&ID_nodes/PR-FIL.png';
import PR_ISO_1 from '../assets/P&ID_nodes/PR-ISO-1.png';
import PR_G from '../assets/P&ID_nodes/PR-G.png';
import PR_PT from '../assets/P&ID_nodes/PR-PT.png';
import PR_REG from '../assets/P&ID_nodes/PR-REG.png';
import PR_VNT from '../assets/P&ID_nodes/PR-VNT.png';
import PR_ISO_2_1 from '../assets/P&ID_nodes/PR-ISO-2-1.png';
import PR_ISO from '../assets/P&ID_nodes/PR-ISO.png';
import BNG from '../assets/P&ID_nodes/BNG.png';
import ORF from '../assets/P&ID_nodes/ORF.png';
import PR_PT_F from '../assets/P&ID_nodes/PR-PT-F.png';
import RV_1 from '../assets/P&ID_nodes/RV-1.png';
import PR_CK from '../assets/P&ID_nodes/PR-CK.png';
import PR_CK_O from '../assets/P&ID_nodes/PR-CK-O.png';

function Pid() {

  onMount(() => {
    const $ = go.GraphObject.make;
    const myDiagram = $(go.Diagram, "myDiagramDiv", {
      initialContentAlignment: go.Spot.Center,
      "undoManager.isEnabled": true
    });


    const myArray = go.Shape.getFigureGenerators().toArray();
    console.log("MY ARRAY");
    myArray.forEach(element => {
      console.log(element.key)
    });
    var KAPPA = 4 * ((Math.sqrt(2) - 1) / 3);

    go.Shape.defineFigureGenerator("HalfEllipse", function(shape, w, h) {
      return new go.Geometry()
            .add(new go.PathFigure(0, 0, true)
                  .add(new go.PathSegment(go.PathSegment.Bezier, w, .5 * h, KAPPA * w, 0, w, (.5 - KAPPA / 2) * h))
                  .add(new go.PathSegment(go.PathSegment.Bezier, 0, h, w, (.5 + KAPPA / 2) * h, KAPPA * w, h).close()))
            .setSpots(0, 0.156, 0.844, 0.844);
    });

    go.Shape.defineFigureGenerator("Curve1", function(shape, w, h) {
      return new go.Geometry()
            .add(new go.PathFigure(0, 0, false)
                  .add(new go.PathSegment(go.PathSegment.Bezier, w, h, KAPPA * w, 0, w, (1 - KAPPA) * h)));
    });

    // go.defineFigureGenerator("HalfEllipse");
    // myDiagram.nodeTemplate =  // provide custom Node appearance
    //   $(go.Node, "Auto",
    //     new go.Binding("location", "pos"),
    //     $(go.Shape,
    //       { figure: "RoundedRectangle",
    //         fill: "white" }),
    //     $(go.TextBlock,
    //       { margin: 5 },
    //       new go.Binding("text", "text"))
    //   );

    // myDiagram.add(
    //   $(go.Part,
    //     $(go.Picture, "valve2.png"),{
    //         // locationSpot: new go.Spot(0.5, 1, 0, -21), locationObjectName: "SHAPE",
    //          rotatable: true, width: 250, height: 250, imageStretch: GraphObject.fill
    //       },
    //   ));

    // myDiagram.add(
    //   $(go.Part,
    //     {
    //       locationSpot: go.Spot.Center,
    //       rotatable: true,
    //     },
    //     $(go.Picture, "valve2.png", {  imageStretch: go.GraphObject.Uniform, width: 100, height: 100 })
    //   )
    // );

    // myDiagram.nodeTemplateMap.add("ImageNode",
    //   $(go.Node, "Spot",
    //     {
    //       // locationSpot: go.Spot.Center,
    //       rotatable: true,
    //       width: 100,
    //       height: 100,
    //       // location: new go.Binding("location", "pos", go.Point.parse).makeTwoWay(go.Point.stringify)
    //     },
    //     new go.Binding("position", "pos", go.Point.parse).makeTwoWay(go.Point.stringify),
    //     $(go.Picture, { imageStretch: go.GraphObject.Uniform },
    //       new go.Binding("img", "img")),
    //     $(go.TextBlock,
    //       new go.Binding("text", "text"))
    //   )
    // );

    myDiagram.nodeTemplateMap.add("ImageNode",
      $(go.Node, "Vertical",  // Use a Vertical panel to stack the elements
        {
          rotatable: true,
          width: 100,
          height: 100,
        },
        new go.Binding("position", "pos", go.Point.parse).makeTwoWay(go.Point.stringify),
        $(go.Picture, 
          {
            imageStretch: go.GraphObject.Uniform,
            width: 100,  // Set width of the picture
            height: 80   // Set height of the picture to leave space for the text
          },
          new go.Binding("source", "source")),
        $(go.TextBlock,
          {
            margin: new go.Margin(4, 0, 0, 0),  // Optional: add some margin to the text
            width: 100,  // Ensure the text block has the same width as the picture
            textAlign: "center"  // Optional: center-align the text
          },
          new go.Binding("text", "text"))
      )
    );



    myDiagram.nodeTemplateMap.add("Node",  // provide custom Node appearance
      $(go.Node, "Vertical",
        new go.Binding("location", "pos"),
        
          $(go.Shape,
          { figure: "RoundedRectangle",
            fill: "white" ,
            height: 2}),
        $(go.TextBlock, "Auto",
          { margin: 5 },
          new go.Binding("text", "text"))
      ));

      myDiagram.nodeTemplateMap.add("Valve",
        $(go.Node, "Vertical",
          {
            // locationSpot: new go.Spot(0.5, 1, 0, -21), locationObjectName: "SHAPE",
            locationObjectName: "SHAPE",
            selectionObjectName: "SHAPE", rotatable: true
          },
          new go.Binding("angle").makeTwoWay(),
          // new go.Binding("location", "pos", go.Point.parse).makeTwoWay(go.Point.stringify),
          new go.Binding("location", "pos"),
          new go.Binding("fromSpot", "fromSpot"),
          new go.Binding("toSpot", "toSpot"),
          $(go.TextBlock,
            { alignment: go.Spot.Center, textAlign: "center", margin: 5, editable: true },
            new go.Binding("text").makeTwoWay(),
            // keep the text upright, even when the whole node has been rotated upside down
            new go.Binding("angle", "angle", a => a === 180 ? 180 : 0).ofObject()),
          $(go.Shape,
            {
              name: "SHAPE",
              geometryString: "F1 M0 0 L40 20 40 0 0 20z M20 10 L20 30 M12 30 L28 30",
              strokeWidth: 2,
              // fill: $(go.Brush, "Linear", { 0: "gray", 0.35: "white", 0.7: "gray" }),
              // portId: "", fromSpot: new go.Spot(1, 0.35), toSpot: new go.Spot(0, 0.35)
              // fill: "GoldenRod"
            }, new go.Binding("fill", "fill"))
            
        ));

        myDiagram.nodeTemplateMap.add("Bottle",
          $(go.Node, "Vertical",
          {
            rotatable: true,
          },
          new go.Binding("position", "pos", go.Point.parse).makeTwoWay(go.Point.stringify),
          new go.Binding("fromSpot", "fromSpot"),
          new go.Binding("toSpot", "toSpot"),
            $(go.Shape,
            {
              figure: "HalfEllipse",
              fill: "ForestGreen",
              angle: -90,
              width: 40,
              height: 40
            }),
            $(go.Shape,
            {
              figure: "HalfEllipse",
              fill: "ForestGreen",
              angle: -90,
            }),
        ))

        myDiagram.nodeTemplateMap.add("FIL",
          $(go.Node, "Vertical",
          {
            rotatable: true
          },
            $(go.Shape,
            {
              figure: "None",
              fill: "white",
              width: 40,
              height: 40,
              angle: 45,
            }),
            $(go.Shape,
            {
              figure: "LineV",
              fill: "black",
              strokeDashArray: [10, 10]
            }),
            $(go.TextBlock,
            new go.Binding("text", "text"))
        ))

        myDiagram.nodeTemplateMap.add("FIL",
          $(go.Node, "Vertical",
          {
            rotatable: true
          },
          $(go.Panel, "Auto",
          $(go.Shape,
            {
              figure: "None",
              fill: "white",
              width: 40,
              height: 40,
              angle: 45,
            }),
            $(go.Shape,
            {
              figure: "LineV",
              fill: "black",
              strokeDashArray: [10, 10]
            }),
          ),
            
          $(go.TextBlock,
          new go.Binding("text", "text"))
        ))

        // myDiagram.nodeTemplateMap.add("PT",
        //   $(go.Node, "Auto",
        //   {
        //     rotatable: true
        //   },
        //     $(go.Shape,
        //     {
        //       figure: "Circle",
        //       fill: "white",
        //       width: 20,
        //       height: 20,
        //     }),
        //     $(go.TextBlock,
        //     {
        //       text: "P"
        //     }),
        // ))

        myDiagram.nodeTemplateMap.add("PT",
          $(go.Node, "Vertical",
          {
            // fromSpot: go.Spot.Center,  // coming out from middle-right
            // toSpot: go.Spot.LeftCenter, 
            // fromSpot: new go.Spot(0.5, 0), toSpot: new go.Spot(0, 0),
            // fromSpot: new go.Spot(0.3, 0.3), toSpot: new go.Spot(0.8, 0.3),
            // fromEndSegmentLength: 0, toEndSegmentLength: 0,
            rotatable: true
          },
          new go.Binding("fromSpot", "fromSpot"),
          new go.Binding("toSpot", "toSpot"),
          $(go.Shape,
            {figure: "LineV",
            height: 30,
          }
          ),
            $(go.Panel, "Auto",
            $(go.Shape,
            {
              figure: "Circle",
              fill: "white",
              width: 20,
              height: 20,
            }),
            $(go.TextBlock,
            {
              text: "P"
            }),
            ),

            $(go.TextBlock,
            new go.Binding("text", "text"))
            
        ))

        myDiagram.nodeTemplateMap.add("ORF",
          $(go.Node, "Vertical",
          {
            rotatable: true
          },
            $(go.Shape,
            {
              figure: "Curve1",
              // fill: "white",
              width: 30,
              height: 20,
              angle: 150
            }),
            $(go.Shape,
            {
              figure: "Curve1",
              width: 30,
              height: 20,
              angle: -25
            }),
            $(go.TextBlock,
            new go.Binding("text", "text"))
        ))

      myDiagram.linkTemplate =
        $(go.Link,
          { routing: go.Link.AvoidsNodes, curve: go.Link.None, corner: 10, reshapable: true },
          new go.Binding("points").makeTwoWay(),
          // mark each Shape to get the link geometry with isPanelMain: true

          //THIS CREATES MULTIPLE LAYERS THAT CREATE THE ULTIMATE LINK
          $(go.Shape, { isPanelMain: true, stroke: "ForestGreen", strokeWidth: 7 }),
          //$(go.Shape, { isPanelMain: true, stroke: "gray", strokeWidth: 5 }),
          //$(go.Shape, { isPanelMain: true, stroke: "white", strokeWidth: 3, name: "PIPE", strokeDashArray: [10, 10] }),
          //$(go.Shape, { toArrow: "Triangle", scale: 1.3, fill: "gray", stroke: "black" })
        );


    var nodeDataArray = [
      { key: "B1", category: "Bottle", fromSpot: go.Spot.TopCenter, toSpot: go.Spot.TopCenter, pos: "-4000 400" },
      { key: "B2", category: "Bottle", fromSpot: go.Spot.TopCenter, toSpot: go.Spot.Left, pos: "-3880 400" },
      { key: "B3", category: "Bottle", fromSpot: go.Spot.TopCenter, toSpot: go.Spot.TopCenter, pos: "-3760 400" },
      // { key: "FIL1", category: "FIL", text: "PR-FIL" },
      // { key: "PR-ISO1", category: "Valve", angle: 180, text: "PR-ISO1", fill: "ForestGreen" },
      // { key: "PR-PT-1", category: "PT", text: "PR-PT-1", fromSpot: go.Spot.RightCenter, toSpot: go.Spot.LeftCenter },
      // { key: "PR-PT-2", category: "PT", text: "PR-PT-2", fromSpot: go.Spot.RightCenter, toSpot: go.Spot.LeftCenter },
      // { key: "PR-VNT", category: "Valve", angle: 90, text: "PR-VNT", fill: "White", fromSpot: go.Spot.LeftCenter, toSpot: go.Spot.LeftCenter },
      // { key: "PR-ISO-F", category: "Valve", angle: 180, text: "PR-ISO-F", fill: "GoldenRod" },
      // { key: "PR-ISO-O", category: "Valve", angle: 180, text: "PR-ISO-O", fill: "GoldenRod" },
      // { key: "F-ORF", category: "ORF", text: "F-ORF"},
      // { key: "O-ORF", category: "ORF", text: "O-ORF"},
      // { key: "PR-PT-F", category: "PT", text: "PR-PT-F" },
      // { key: "PR-PT-O", category: "PT", text: "PR-PT-O" },
      {key: "PR-FIL", category: "ImageNode", source: PR_FIL, pos: "-3880 200", text: "PR-FILT"},
      {key: "PR-ISO-1", category: "ImageNode", source: PR_ISO_1, pos: "-3750 200", text: "PR-ISO-1"},
      // {key: "PR-ISO-2", category: "ImageNode", source: "../assets/P&ID_nodes/PR-ISO-2.png"},
      {key: "PR-G-1", category: "ImageNode", source: PR_G, pos: "-3650 200", text: "PR-G-1"},
      {key: "PR-PT-1", category: "ImageNode", source: PR_PT, pos: "-3550 200", text: "PR-PT-1"},
      {key: "PR-REG", category: "ImageNode", source: PR_REG, pos: "-3450 200", text: "PR-REG"},
      {key: "PR-PT-2", category: "ImageNode", source: PR_PT, pos: "-3350 200", text: "PR-PT-2"},
      {key: "PR-G-2", category: "ImageNode", source: PR_G, pos: "-3250 200", text: "PR-G-2"},
      {key: "PR-VNT", category: "ImageNode", source: PR_VNT, pos: "-3150 200", text: "PR-VNT"},
      {key: "PR-ISO-2", category: "ImageNode", source: PR_ISO_2_1, pos: "-3050 200", text: "PR-ISO-2"},
      {key: "PR-ISO-F", category: "ImageNode", source: PR_ISO, pos: "-2950 0", text: "PR-ISO-F"},
      {key: "F-BNG", category: "ImageNode", source: BNG, pos: "-2800 0", text: "F-BNG"},
      {key: "F-ORF", category: "ImageNode", source: ORF, pos: "-2650 0", text: "F-ORF"},
      {key: "PR-PT-F", category: "ImageNode", source: PR_PT_F, pos: "-2500 0", text: "PR-PT-F"},
      {key: "F-RV-1", category: "ImageNode", source: RV_1, pos: "-2400 0", text: "F-RV-1"},
      {key: "PR-CK-F", category: "ImageNode", source: PR_CK, pos: "-2300 0", text: "PR-CK-F"},
      {key: "PR-ISO-O", category: "ImageNode", source: PR_ISO, pos: "-2950 400", text: "PR-ISO-O"},
      {key: "O-BNG", category: "ImageNode", source: BNG, pos: "-2800 400", text: "O-BNG"},
      {key: "O-ORF", category: "ImageNode", source: ORF, pos: "-2650 400", text: "O-ORF"},
      {key: "PR-PT-O", category: "ImageNode", source: PR_PT_F, pos: "-2500 400", text: "PR-PT-O"},
      {key: "O-RV-1", category: "ImageNode", source: RV_1, pos: "-2400 400", text: "O-RV-1"},
      {key: "PR-CK-O", category: "ImageNode", source: PR_CK_O, pos: "-2300 400", text: "PR-CK-O"},
    ]

    var linkDataArray = [
      { from: "B1", to: "B3" },
      // { from: "B2", to: "FIL1" },
      // { from: "FIL1", to: "PR-ISO1" },
      // { from: "PR-ISO1", to: "PR-PT-1" },
      // { from: "PR-PT-1", to: "PR-PT-2" },
      // { from: "PR-PT-2", to: "PR-VNT" },
      // { from: "PR-VNT", to: "PR-ISO-F" },
      // { from: "PR-VNT", to: "PR-ISO-O" },
      // { from: "PR-ISO-F", to: "F-ORF"},
      // { from: "PR-ISO-O", to: "O-ORF"},
      // { from: "F-ORF", to: "PR-PT-F" },
      // { from: "O-ORF", to: "PR-PT-O" }
      { from: "B2", to: "PR-FIL" },
      { from: "PR-FIL", to: "PR-ISO-1" },
      { from: "PR-ISO-1", to: "PR-G-1" },
      { from: "PR-G-1", to: "PR-PT-1" },
      { from: "PR-PT-1", to: "PR-REG" },
      { from: "PR-REG", to: "PR-PT-2" },
      { from: "PR-PT-2", to: "PR-G-2" },
      { from: "PR-G-2", to: "PR-VNT" },
      { from: "PR-VNT", to: "PR-ISO-2" },
      { from: "PR-ISO-2", to: "PR-ISO-F" },
      { from: "PR-ISO-F", to: "F-BNG" },
      { from: "F-BNG", to: "F-ORF" },
      { from: "F-ORF", to: "PR-PT-F" },
      { from: "PR-PT-F", to: "F-RV-1" },
      { from: "F-RV-1", to: "PR-CK-F" },
      { from: "PR-ISO-2", to: "PR-ISO-O" },
      { from: "PR-ISO-O", to: "O-BNG" },
      { from: "O-BNG", to: "O-ORF" },
      { from: "O-ORF", to: "PR-PT-O" },
      { from: "PR-PT-O", to: "O-RV-1" },
      { from: "O-RV-1", to: "PR-CK-O" },
      { from: "B1", to: "Img1" }

    ]

    // var nodeDataArray = [
    //   { key: "P1", text: "Process", pos: new go.Point(150, 120), category: "Node" },
    //   { key: "P2", text: "Tank", pos: new go.Point(330, 320) },
    //   { key: "V1", text: "V1", pos: new go.Point(270, 120), category: "Valve" },
    //   { key: "P3", text: "Pump", pos: new go.Point(150, 420) },
    //   { key: "V2", text: "VM", pos: new go.Point(150, 280) },
    //   { key: "V3", text: "V2", pos: new go.Point(270, 420) },
    //   { key: "P4", text: "Reserve Tank", pos: new go.Point(450, 140) },
    //   { key: "V4", text: "VA", pos: new go.Point(390, 60) },
    //   { key: "V5", text: "VB", pos: new go.Point(450, 260) },
    //   { key: "B1", category: "Bottle"},
    //   { key: "B2", category: "Bottle"},
    //   { key: "B3", category: "Bottle"},
    //   { key: "F1", category: "Fil"},
    //   { key: "PNPT1", category: "PNPT"},
    //   { key: "ORF1", category: "ORF"}
    // ];

    // var linkDataArray = [
    //   { from: "P1", to: "V1" },
    //   { from: "P3", to: "V2" },
    //   { from: "V2", to: "P1" },
    //   { from: "P2", to: "V3" },
    //   { from: "V3", to: "P3" },
    //   { from: "V1", to: "V4" },
    //   { from: "V4", to: "P4" },
    //   { from: "V1", to: "P2" },
    //   { from: "P4", to: "V5" },
    //   { from: "V5", to: "P2" },
    //   // { from: "B1", to: "B3"}
    // ];

    myDiagram.model = new go.GraphLinksModel(nodeDataArray, linkDataArray);

        // myDiagram.model = new go.Model(
        // [ // for each object in this Array, the Diagram creates a Node to represent it
        //     { key: "Alpha" },
        //     { key: "Beta" },
        //     {key:"P1", "category":"Process", "pos":"150 120", "text":"Process"},
        //     { key: "Gamma" }
        // ]);


    onCleanup(() => {
      myDiagram.div = null;
      });
  });

  return <div class="window-template">
    <div style="height: 60px">
      <GeneralTitleBar name="P&ID"/>
    </div>
    <div class="pid-view">
      <div id="myDiagramDiv" style="width:1536; height:700px; background-color: #DAE4E4;"></div>
    </div>
    <div>
      <Footer />
    </div>
  </div>
}


export default Pid;