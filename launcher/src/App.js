import React from 'react';
import '@fontsource/poppins/300.css';
import '@fontsource/poppins/400.css';
import '@fontsource/poppins/500.css';
import '@fontsource/poppins/700.css';
import '@fontsource/poppins/900.css';
import { Globals } from './global/global-styles';
import Home from './pages/Home';

function App() {
  return (
    <>
      <Globals />
      <Home />
    </>
  );
}

export default App;
