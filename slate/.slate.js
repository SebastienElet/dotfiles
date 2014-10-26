var slate;
slate.configAll({
  "defaultToCurrentScreen": true,
  nudgePercentOf: "screenSize",
  resizePercentOf: "screenSize"
});

// Move + Resize
slate.bind("return:shift;cmd", slate.op(
  "move",
  {
    "x": "screenOriginX",
    "y": "screenOriginY",
    "width": "screenSizeX",
    "height": "screenSizeY"
  }
));

slate.bind("h:shift;cmd", slate.op(
  "push",
  {
    "direction": "left",
    "style": "bar-resize:screenSizeX/2"
  }
));

slate.bind("l:shift;cmd", slate.op(
  "push",
  {
    "direction": "right",
    "style": "bar-resize:screenSizeX/2"
  }
));

slate.bind("j:shift;cmd", slate.op(
  "push",
  {
    "direction": "down",
    "style": "bar-resize:screenSizeY/2"
  }
));

slate.bind("k:shift;cmd", slate.op(
  "push",
  {
    "direction": "up",
    "style": "bar-resize:screenSizeY/2"
  }
));

slate.bind("e:shift;cmd", slate.op(
  "throw",
  {
    "screen": "next"
  }
));

slate.bindAll({
  "h:cmd": slate.op("focus", { "direction" : "left" }),
  "j:cmd": slate.op("focus", { "direction" : "up" }),
  "k:cmd": slate.op("focus", { "direction" : "down" }),
  "l:cmd": slate.op("focus", { "direction" : "right" }),
  "r:shift;cmd": slate.op("relaunch")
});
