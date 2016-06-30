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

}


function sortBySize() {

}


window.onload = function () {
    document.getElementById("hname").onclick = sortByName;
}
