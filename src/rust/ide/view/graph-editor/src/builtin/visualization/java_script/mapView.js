function loadScript(url) {
    var script = document.createElement("script");
    script.src = url;

    document.head.appendChild(script);
}

function loadStyle(url) {
    var link  = document.createElement("link");
    link.href = url;
    link.rel  = 'stylesheet';

    document.head.appendChild(link);
}

loadScript('https://unpkg.com/deck.gl@latest/dist.min.js');
loadScript('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.js');
loadStyle('https://api.tiles.mapbox.com/mapbox-gl-js/v1.6.1/mapbox-gl.css');


/**
 * Provides a deck.gl-based map visualization for IDE.
 *
 * It can be created from passed JS object.
 *
 * > Example creates a map with described properties with a scatter plot overlay.
 * {
 *  latitude: 37.8,
 *  longitude: -122.45,
 *  zoom: 15,
 *  controller: true,
 *  layers: [
 *    new deck.ScatterplotLayer({
 *      data: [
 *        {position: [-122.45, 37.8], color: [255, 0, 0], radius: 100}
 *      ],
 *      getColor: d => d.color,
 *      getRadius: d => d.radius
 *    })
 *  ]
 * }
 */
class MapViewVisualization extends Visualization {
    static inputType = "Any"

    onDataReceived(data) {
        this.setPreprocessor("None");

        while (this.dom.firstChild) {
            this.dom.removeChild(this.dom.lastChild);
        }

        const styleHead = document.createElement("style")
        styleHead.innerText = `.mapboxgl-map {
            border-radius: 14px;
        }`
        document.head.appendChild(styleHead);

        const width   = this.dom.getAttributeNS(null, "width");
        const height  = this.dom.getAttributeNS(null, "height");
        const mapElem = document.createElement("div");
        mapElem.setAttributeNS(null,"id"   , "map");
        mapElem.setAttributeNS(null,"style","width:" + width + "px;height: " + height + "px;");
        this.dom.appendChild(mapElem);

        const parsedData = eval('('+data+')' );

        // TODO [MM] : Change default map style to Light when light mode lands on main.
        const deckgl = new deck.DeckGL({
            container: 'map',
            mapboxApiAccessToken: 'pk.eyJ1IjoidWJlcmRhdGEiLCJhIjoiY2pudzRtaWloMDAzcTN2bzN1aXdxZHB5bSJ9.2bkj3IiRC8wj3jLThvDGdA',
            mapStyle: parsedData.mapStyle || 'mapbox://styles/mapbox/dark-v9',
            initialViewState: {
                longitude: parsedData.longitude || 0.0,
                latitude: parsedData.latitude || 0.0,
                zoom: parsedData.zoom || 3,
                pitch: parsedData.pitch || 0
            },
            controller: parsedData.controller || false
        });

        deckgl.setProps({
            layers: parsedData.layers || []
        });
    }

    setSize(size) {
        this.dom.setAttributeNS(null, "width", size[0]);
        this.dom.setAttributeNS(null, "height", size[1]);
    }
}

return MapViewVisualization;
