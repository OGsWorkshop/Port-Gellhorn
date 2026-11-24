// Collects the prebuilt runtime into launcher/runtime-bundle (packed into the exe as resources/runtime).
// Override sources with XCL_SRC (folder with xcl.exe + D3D12\) and DXC_SRC (folder with dxcompiler.dll + dxil.dll).
const fs = require('fs');
const path = require('path');
const XCL = process.env.XCL_SRC || 'M:/GTA6/xcl-test';
const DXC = process.env.DXC_SRC || 'C:/Program Files (x86)/Windows Kits/10/bin/10.0.26100.0/x64';
const out = path.join(__dirname, '..', 'runtime-bundle');
fs.rmSync(out, { recursive: true, force: true });
fs.mkdirSync(path.join(out, 'D3D12'), { recursive: true });
const cp = (src, dst) => {
  if (!fs.existsSync(src)) throw new Error('missing ' + src);
  fs.copyFileSync(src, path.join(out, dst));
};
cp(path.join(XCL, 'xcl.exe'), 'xcl.exe');
cp(path.join(XCL, 'D3D12', 'D3D12Core.dll'), 'D3D12/D3D12Core.dll');
cp(path.join(XCL, 'D3D12', 'd3d12SDKLayers.dll'), 'D3D12/d3d12SDKLayers.dll');
cp(path.join(DXC, 'dxcompiler.dll'), 'dxcompiler.dll');
cp(path.join(DXC, 'dxil.dll'), 'dxil.dll');
console.log('runtime staged in', out);
