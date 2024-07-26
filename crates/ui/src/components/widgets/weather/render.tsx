import { useEffect, useState } from "react";

import React from "react";

export default function Render() {
  const [data, setData] = useState<any>();

  async function fetchData() {
    const response = await fetch("https://wttr.in/Cuiaba?format=j1", {
      method: "GET",
    });

    setData(await response.json());
  }
  //www.terabyteshop.com.br/produto/25612/microfone-gamer-condensador-fifine-superframe-edition-sfm2-usb-rgb-black

  https: useEffect(() => {
    fetchData();
  }, []);

  return <></>;
}
