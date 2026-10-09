import QQ

Scene {
    name: "sample"
    skyTop: "#0b1430"
    skyHorizon: "#c7623a"
    groundHorizon: "#2a1a1a"
    groundBottom: "#07080c"
    skyLight: 0.75
    exposure: 1
    bloom: true

    Sun {
        name: "Sun"
        elevation: 38
        azimuth: -35
        color: "#fff1dc"
        brightness: 1.15
    }

    Ground {
        name: "Ground"
        extent: 40
        colour: "#151821"
    }

    Shape {
        name: "Plinth"
        position: Qt.vector3d(0, 0.25, 0)
        eulerRotation: Qt.vector3d(0, 15, 0)
        scale: Qt.vector3d(2, 0.5, 2)
        form: "cube"
        colour: "#2b3140"
        metalness: 0.2
        roughness: 0.6
        emissive: "#000000"
        emissivePower: 0
    }

    Lamp {
        name: "Ember lamp"
        position: Qt.vector3d(1.5, 2.2, 1)
        color: "#ffb15c"
        brightness: 4
        reach: 6
    }

    Prop {
        name: "Orb"
        position: Qt.vector3d(0, 1.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../orb.gltf"
    }
}
