const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('scarlett', {
  getState: () => ipcRenderer.invoke('state:get'),
  pickGame: () => ipcRenderer.invoke('game:pick'),
  pickRepo: () => ipcRenderer.invoke('repo:pick'),
  setOptions: (opts) => ipcRenderer.invoke('options:set', opts),
  setup: () => ipcRenderer.invoke('run:setup'),
  play: (opts) => ipcRenderer.invoke('run:play', opts),
  stop: () => ipcRenderer.invoke('run:stop'),
  openExternal: (url) => ipcRenderer.invoke('open:external', url),
  minimize: () => ipcRenderer.invoke('win:minimize'),
  close: () => ipcRenderer.invoke('win:close'),
  onLog: (cb) => {
    const h = (_e, msg) => cb(msg);
    ipcRenderer.on('log', h);
    return () => ipcRenderer.removeListener('log', h);
  },
  onRunState: (cb) => {
    const h = (_e, msg) => cb(msg);
    ipcRenderer.on('run-state', h);
    return () => ipcRenderer.removeListener('run-state', h);
  },
});
