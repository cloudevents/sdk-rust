import $ from 'jquery';

import 'bootstrap';
import 'bootstrap/dist/css/bootstrap.min.css';

import("./pkg").then(rustModule => {
    $(document).ready(function () {
        $("#send").click(function () {
            let target = $("#event_target").val()
            let ty = $("#event_type").val()
            let dataContentType = $("#event_datacontenttype").val()
            let data = $("#event_data").val()

            rustModule.run(target, ty, dataContentType, data).catch(console.error);
        });
    })
}).catch(console.error);