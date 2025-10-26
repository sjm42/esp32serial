// form.js for esp32serial

document.addEventListener("DOMContentLoaded", function () {
    document.querySelector("form[name='esp32cfg']")
        .addEventListener("submit", handleCfgSubmit);
});

const handleCfgSubmit = async (event) => {
    event.preventDefault();
    const form = event.currentTarget;
    const url = form.action;

    try {
        const formData = new FormData(form);
        const responseData = await postCfgDataAsJson({url, formData});
        console.log({responseData});
    } catch (error) {
        console.error(error);
    }
}

var postCfgDataAsJson = async ({url, formData}) => {
    let formObj = Object.fromEntries(formData.entries());
    // convert integers
    formObj.port = parseInt(formObj.port);
    formObj.v4mask = parseInt(formObj.v4mask);
    formObj.bps = parseInt(formObj.bps);
    formObj.serial_tcp_port = parseInt(formObj.serial_tcp_port);
    // convert booleans
    formObj.wifi_wpa2ent = (formObj.wifi_wpa2ent === "on");
    formObj.v4dhcp = (formObj.v4dhcp === "on");
    formObj.serial_write_enabled = (formObj.serial_write_enabled === "on");
    // serialize to JSON
    const formDataJsonString = JSON.stringify(formObj);

    const fetchOptions = {
        method: "POST",
        mode: 'cors',
        keepalive: false,
        headers: {'Accept': 'application/json', 'Content-Type': 'application/json'},
        body: formDataJsonString
    };
    const response = await fetch(url, fetchOptions);

    if (!response.ok) {
        const errorMessage = await response.text();
        throw new Error(errorMessage);
    }

    return response.json();
}
// EOF
