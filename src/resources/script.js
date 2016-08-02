var nameSorted = false;
var dateSorted = false;
var sizeSorted = false;


function sortByName() {
    var table = document.getElementById("file-table");
    var rows  = Array.prototype.slice.call(table.rows, 1);

    if (nameSorted) {
        rows.reverse();
    } else {
        rows.sort(function (a, b) {
            return a.cells[0].innerHTML.localeCompare(b.cells[0].innerHTML);
        });
        nameSorted = true;
        dateSorted = false;
        sizeSorted = false;
    }
    for (var i = 1; i < rows.length; i++) {
        table.appendChild(rows[i]);
    }
}

function sortByDate() {
    var table = document.getElementById("file-table");
    var rows  = Array.prototype.slice.call(table.rows, 1);

    if (dateSorted) {
        rows.reverse();
    } else {
        rows.sort(function (a, b) {
            var size_a = parseInt(a.cells[1].getAttribute("time"));
            var size_b = parseInt(b.cells[1].getAttribute("time"));
            return size_a - size_b;
        });
        nameSorted = false;
        dateSorted = true;
        sizeSorted = false;
    }
    for (var i = 1; i < rows.length; i++) {
        table.appendChild(rows[i]);
    }
}

function sortBySize() {
    var table = document.getElementById("file-table");
    var rows  = Array.prototype.slice.call(table.rows, 1);

    if (sizeSorted) {
        rows.reverse();
    } else {
        rows.sort(function (a, b) {
            var size_a = parseInt(a.cells[2].getAttribute("data-size"));
            var size_b = parseInt(b.cells[2].getAttribute("data-size"));
            return size_a - size_b;
        });
        nameSorted = false;
        dateSorted = false;
        sizeSorted = true;
    }
    for (var i = 1; i < rows.length; i++) {
        table.appendChild(rows[i]);
    }
}

function deleteResource(event) {
    var http = new XMLHttpRequest();
    http.onreadystatechange = function() {
        if (http.readyState == 4 && http.status == 200) {
            location.reload(true);
        }
    }
    var t = event.target;
    http.open("DELETE",
              t.attributes.getNamedItem("res").value,
              true);

    http.send(null);
}

window.onload = function () {
    document.getElementById("hname").onclick = sortByName;
    document.getElementById("hsize").onclick = sortBySize;
    document.getElementById("hmodified").onclick = sortByDate;

    var elements = document.getElementsByClassName('delete-button');

    for (var i = 0; i < elements.length; i++) {
        elements[i].onclick = deleteResource;
    }
}
