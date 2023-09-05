import { Line, Arc, Bezier, Square, Circle, strokeLight, strokeDefault } from './shapes.js';
import { snapToGrid, zero, isBoxInside, sub } from './math.js';

class PlayingArea {
    constructor(window, canvas, canvasContainer) {
        this.window = window;
        this.container = canvasContainer;
        this.canvas = canvas;
        this.ctx = this.canvas.getContext("2d");
        this.canvasCursor = cursorNormal;

        // Canvas parameters
        this.gridSpacing = 10;  // 10mm
        this.axisColor = "#000";
        this.gridColor = "#eee";
        this.headPosition = { x: 10, y: 10 };

        // Zoom 
        this.mouseIsDown = false;
        this.prevAbolute = zero();
        this.prevCursor = zero();
        this.scale = 1;
        this.offsetX = 0;
        this.offsetY = 0;

        // Event listeners
        this.window.addEventListener('resize', this.onResizeWindow.bind(this));
        this.canvas.addEventListener("wheel", this.onWheel.bind(this));
        this.canvas.addEventListener("mousedown", this.onMouseDown.bind(this));
        this.canvas.addEventListener("mouseup", this.onMouseUp.bind(this));
        this.canvas.addEventListener("mousemove", this.onMouseMove.bind(this));
        this.canvas.addEventListener("keydown", this.handleKeyDown.bind(this));
        this.canvas.addEventListener('mouseenter', function () {
            switch (editorState) {
                case 'pointer':
                    this.canvasCursor = cursorNormal;
                    break;
                case 'selection':
                    this.canvasCursor = cursorCrossHair;
                    break;
                case 'drawLine':
                    this.canvasCursor = cursorEditLine;
                    break;
                case 'drawArc':
                    this.canvasCursor = cursorEditArc;
                    break;
                case 'drawBezier':
                    this.canvasCursor = cursorEditBezier;
                    break;
                case 'drawSquare':
                    this.canvasCursor = cursorEditSquare;
                    break;
                case 'drawCircle':
                    this.canvasCursor = cursorEditCircle;
                    break;
                default:
                    this.canvasCursor = cursorCrossHair;
                    break;
            }
            this.canvas.cursor = this.canvasCursor;
        }.bind(this));
        this.canvas.addEventListener('mouseleave', function () {
            this.canvasCursor = 'default';  // Reset to default cursor
        }.bind(this));
        this.canvas.addEventListener('contextmenu', (e) => {
            e.preventDefault();
            // Hack to have always good contextMenu dimensions
            contextMenu.style.opacity = '0';
            contextMenu.style.display = 'block';
            let menuWidth = contextMenu.offsetWidth;
            let menuHeight = contextMenu.offsetHeight;
            contextMenu.style.display = 'none';
            contextMenu.style.opacity = '';

            const canvasBounds = canvas.getBoundingClientRect();
            const x = e.clientX - canvasBounds.left;
            const y = e.clientY - canvasBounds.top;

            // Set the default position to the right and below the cursor
            let posX = x;
            let posY = y;

            // If clicking near the right edge, display to the left of the cursor
            if (x + menuWidth > canvasBounds.right - canvasBounds.left) {
                posX = x - menuWidth;
            }

            // If clicking near the bottom edge, display above the cursor
            if (y + menuHeight > canvasBounds.bottom) {
                posY = y - menuHeight;
            }

            // Adjust the context menu position
            contextMenu.style.left = posX + 'px';
            contextMenu.style.top = posY + 'px';
            contextMenu.style.display = 'block';
        });
        document.getElementById("apply-settings").addEventListener('click', this.applyCanvasSettings.bind(this));
        document.getElementById("icon-cog").addEventListener('click', this.toggleSettings.bind(this));

        this.shapes = [];
        this.currentShape = undefined;
        this.selectionArea = { bl: zero(), tr: zero() };

        // Few examples
        let line = new Line(this.ctx, { x: 300, y: 600 }, { x: 550, y: 800 });
        this.shapes.push(line);

        // let arc = new Arc(this.ctx, { x: 400, y: 400 }, { x: 450, y: 430 });
        // this.shapes.push(arc);

        let bezier = new Bezier(this.ctx, zero(), { x: 100, y: -150 },
            { x: 200, y: 50 }, { x: 500, y: 100 });
        bezier.move({ x: 500, y: 500 });
        this.shapes.push(bezier);

        for (const shape of this.shapes) {
            shape.removeSelection();
        }

        this.onResizeWindow();
        this.canvas.focus();

        document.getElementById('status-gridsize').textContent = "Grid size: " + this.gridSpacing + "mm";
    }
    handleKeyDown(e) {
        console.log(e.key);
        if (e.key === "Delete" || e.key === "Backspace") {
            this.shapes = this.shapes.filter(shape => !shape.hasSelection());
            this.render();
        }
        if (e.key === "Shift") {
            if (editorState === 'pointer') {
                this.goToselectionMode();
            }
        }
    }
    getRelativePositionCursor(cursor) {
        const rect = this.canvas.getBoundingClientRect();
        const clickX = cursor.x - rect.left - this.offsetX;
        const clickY = cursor.y - rect.top - this.offsetY;
        const relativeX = clickX / this.scale;
        const relativeY = this.canvas.height - clickY / this.scale;
        return { x: relativeX, y: relativeY };
    }
    onResizeWindow() {
        this.offsetY = -(this.canvas.height - (window.innerHeight - 60)) * this.scale;
        this.render();
    }
    onWheel(event) {
        const zoomFactor = 0.1;
        let newScale;

        // Get mouse position relative to the canvas
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = event.clientX - rect.left;
        const mouseY = event.clientY - rect.top;

        // Compute the transformation center: We compute the current mouse position in "world coordinates"
        const worldX = (mouseX - this.offsetX) / this.scale;
        const worldY = (mouseY - this.offsetY) / this.scale;

        if (event.deltaY < 0) {
            // Zoom in
            newScale = this.scale * (1 + zoomFactor);
            if (newScale > 10) {
                newScale = 10;
            }
        } else {
            // Zoom out
            newScale = this.scale / (1 + zoomFactor);
            if (newScale < 1) {
                newScale = 1;
            }
        }

        // Compute the new offset after the scaling: The idea here is to first apply the scaling transformation 
        // and then translate the world so that the point under the mouse remains in the same place
        this.offsetX = mouseX - worldX * newScale;
        this.offsetY = mouseY - worldY * newScale;

        this.scale = newScale;

        this.render();
        event.preventDefault();
    }
    goToselectionMode() {
        let pointerIcon = document.getElementById('icon-selection');
        icons.forEach(i => i.classList.remove("icon-selected"));
        pointerIcon.classList.add("icon-selected");
        editorState = 'selection';
        this.canvasCursor = cursorNormal;
        canvas.style.cursor = cursorNormal;
    }
    goToPointerMode() {
        let pointerIcon = document.getElementById('icon-pointer');
        icons.forEach(i => i.classList.remove("icon-selected"));
        pointerIcon.classList.add("icon-selected");
        editorState = 'pointer';
        this.canvasCursor = cursorNormal;
        canvas.style.cursor = cursorNormal;
    }
    onMouseDown(event) {
        let start;
        if (event.button === 0) {
            this.mouseIsDown = true;
            this.prevAbolute = { x: event.clientX, y: event.clientY };
            let cursor = this.getRelativePositionCursor({ x: event.clientX, y: event.clientY });
            this.prevCursor = cursor;
            switch (editorState) {
                case 'pointer':
                    for (const shape of this.shapes) {
                        shape.setSelection(cursor);
                    }
                    break;

                case 'drawLine':
                    for (const shape of this.shapes) {
                        shape.removeSelection();
                    }
                    start = snapToGrid(cursor, this.gridSpacing);
                    this.currentShape = new Line(this.ctx, start, start);
                    break;

                case 'drawArc':
                    for (const shape of this.shapes) {
                        shape.removeSelection();
                    }
                    start = snapToGrid(cursor, this.gridSpacing);
                    this.currentShape = new Arc(this.ctx, start, start)
                    break;

                case 'drawBezier':
                    for (const shape of this.shapes) {
                        shape.removeSelection();
                    }
                    start = snapToGrid(cursor, this.gridSpacing);
                    this.currentShape = new Bezier(this.ctx, start, start, start, start);
                    break;

                case 'drawSquare':
                    for (const shape of this.shapes) {
                        shape.removeSelection();
                    }
                    start = snapToGrid(cursor, this.gridSpacing);
                    this.currentShape = new Square(this.ctx, start, zero());
                    break;

                case 'drawCircle':
                    for (const shape of this.shapes) {
                        shape.removeSelection();
                    }
                    start = snapToGrid(cursor, this.gridSpacing);
                    this.currentShape = new Circle(this.ctx, start, zero());
                    break;

                case 'selection':
                    this.selectionArea = {
                        bl: { x: cursor.x, y: cursor.y },
                        tr: { x: cursor.x, y: cursor.y },
                    };
                    break;

                default:
                    break;
            }
            this.render();
        }
    }
    onMouseUp(event) {
        if (event.button === 0) {
            this.mouseIsDown = false;
            let cursor = this.getRelativePositionCursor({ x: event.clientX, y: event.clientY });
            switch (editorState) {
                case 'pointer':
                    for (const shape of this.shapes) {
                        if (shape.selection > -2)
                            shape.snap(this.gridSpacing);
                    }
                    break;

                case 'drawLine':
                case 'drawArc':
                case 'drawBezier':
                case 'drawSquare':
                case 'drawCircle':
                    this.currentShape.snap(this.gridSpacing);
                    if (this.currentShape.valid()) {
                        this.shapes.push(this.currentShape);
                    }
                    this.currentShape = undefined;
                    this.goToPointerMode();
                    break;

                case 'selection':
                    const bl = this.selectionArea.bl;
                    const tr = this.selectionArea.tr;
                    console.log(this.selectionArea);
                    if (tr.x > bl.x) {
                        for (const shape of this.shapes) {
                            const bb = shape.getBoundingBox();

                            if (isBoxInside(this.selectionArea, bb)) {
                                shape.selection = -1;
                            }
                        }
                    }
                    this.selectionArea = { bl: zero(), tr: zero() };
                    this.goToPointerMode();
                    break;

                default:
                    break;
            }
        }
        this.onMouseMove(event);
    }
    onMouseMove(event) {
        this.canvas.style.cursor = this.canvasCursor;
        const deltaAbsoluteX = event.clientX - this.prevAbolute.x;
        const deltaAbsoluteY = event.clientY - this.prevAbolute.y;
        let cursor = this.getRelativePositionCursor({ x: event.clientX, y: event.clientY });
        const deltaCursor = sub(cursor, this.prevCursor);

        if (this.mouseIsDown) {
            switch (editorState) {
                case 'pointer':
                    let somethingSelected = false;
                    for (const shape of this.shapes) {
                        if (shape.selection > -2) {
                            shape.modify(cursor, deltaCursor);
                            somethingSelected = true;
                        }
                    }
                    // move the canvas if no object was selected
                    if (!somethingSelected) {
                        // Calculate how far the cursor has moved
                        if (Math.abs(deltaAbsoluteX) > 3 || Math.abs(deltaAbsoluteY) > 3) {
                            this.modifyingShape = true;
                        }
                        // Adjust offsets by this distance
                        this.offsetX += deltaAbsoluteX;
                        this.offsetY += deltaAbsoluteY;
                    }
                    break;

                case 'drawLine':
                case 'drawArc':
                case 'drawBezier':
                case 'drawSquare':
                case 'drawCircle':
                    this.currentShape.modify(cursor, deltaCursor);
                    break;

                case 'selection':
                    this.selectionArea.tr.x += deltaCursor.x;
                    this.selectionArea.tr.y += deltaCursor.y;
                    break;

                default:
                    this.canvas.style.cursor = this.canvasCursor;
                    break;
            }
        }
        // Update previous cursor position for next movement calculation
        this.prevAbolute = { x: event.clientX, y: event.clientY };
        this.prevCursor = cursor;
        this.render();
    }
    render() {
        this.ctx.setTransform(1, 0, 0, 1, 0, 0);
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.setTransform(this.scale, 0, 0, this.scale, this.offsetX, this.offsetY);
        // Set the origin to the bottom left
        this.ctx.translate(0, this.canvas.height);  // Translate by the canvas height
        this.ctx.scale(1, -1);  // Flip vertically

        this.drawAll();
    }
    drawAll() {
        this.drawGrid();
        this.drawHeadPosition();
        this.drawContent();
    }
    drawGrid() {
        this.ctx.lineWidth = 1;
        this.ctx.strokeStyle = this.gridColor;
        // Vertical grid lines
        for (let x = 0; x <= this.canvas.width; x += this.gridSpacing) {
            this.ctx.beginPath();
            this.ctx.moveTo(x, 0);
            this.ctx.lineTo(x, this.canvas.height);
            this.ctx.stroke();
        }

        // Horizontal grid lines
        for (let y = 0; y <= this.canvas.height; y += this.gridSpacing) {
            this.ctx.beginPath();
            this.ctx.moveTo(0, y);
            this.ctx.lineTo(this.canvas.width, y);
            this.ctx.stroke();
        }
    }
    drawHeadPosition() {
        const circleRadius = 10; // Adjust as needed
        const crossLength = 15;  // Adjust as needed

        // Draw circle
        this.ctx.beginPath();
        this.ctx.arc(this.headPosition.x, this.headPosition.y, circleRadius, 0, 2 * Math.PI);
        this.ctx.fillStyle = "#000"; // Black color for the circle
        this.ctx.fill();
        this.ctx.closePath();

        // Draw the rotated cross
        this.ctx.strokeStyle = "#FFF"; // White color for the cross to contrast with the black circle
        this.ctx.lineWidth = 2;        // Adjust as needed



        // Vertical line of the cross
        this.ctx.beginPath();
        this.ctx.moveTo(this.headPosition.x, -crossLength / 2 + this.headPosition.y);
        this.ctx.lineTo(this.headPosition.x, crossLength / 2 + this.headPosition.y);
        this.ctx.stroke();

        // Horizontal line of the cross
        this.ctx.beginPath();
        this.ctx.moveTo(this.headPosition.x - crossLength / 2, this.headPosition.y);
        this.ctx.lineTo(this.headPosition.x + crossLength / 2, this.headPosition.y);
        this.ctx.stroke();
    }
    drawContent() {
        for (const shape of this.shapes) {
            shape.draw();
        }
        if (this.currentShape !== undefined)
            this.currentShape.draw();

        const bl = this.selectionArea.bl;
        const tr = this.selectionArea.tr;
        if (bl.x !== tr.x && bl.y !== tr.y) {
            let p = new Path2D();
            this.ctx.strokeStyle = strokeLight;
            this.ctx.setLineDash([3, 3]);
            p.moveTo(bl.x, bl.y);
            p.lineTo(bl.x, tr.y);
            p.lineTo(tr.x, tr.y);
            p.lineTo(tr.x, bl.y);
            p.lineTo(bl.x, bl.y);
            this.ctx.stroke(p);
            this.ctx.setLineDash([]);
            this.ctx.strokeStyle = strokeDefault;
        }
    }
    toggleSettings() {
        const settingsPanel = document.getElementById("settingsPanel");
        const leftPanel = document.getElementById("left-panel");
        const canvasWidthInput = document.getElementById("canvasWidthInput");
        const canvasHeightInput = document.getElementById("canvasHeightInput");
        const canvas = document.getElementById("myCanvas");

        // If the settings panel is not visible, show it and hide the left panel
        if (settingsPanel.style.display === "none" || settingsPanel.style.display === "") {
            settingsPanel.style.display = "block";
            leftPanel.style.display = "none";

            // Set the current canvas width and height values in the input fields
            canvasWidthInput.value = canvas.width;
            canvasHeightInput.value = canvas.height;
        } else {
            settingsPanel.style.display = "none";
            leftPanel.style.display = "block";
        }
    }
    applyCanvasSettings() {
        const canvasWidthInput = document.getElementById("canvasWidthInput");
        const canvasHeightInput = document.getElementById("canvasHeightInput");
        const canvas = document.getElementById("myCanvas");

        canvas.width = canvasWidthInput.value;
        canvas.height = canvasHeightInput.value;

        this.toggleSettings(); // Hide the settings panel
    }
    // Destructor
    destroy() {
        this.window.removeEventListener('resize', this.onResizeWindow);
        this.canvas.removeEventListener("wheel", this.onWheel);
        this.canvas.removeEventListener("mousedown", this.onMouseDown);
        this.window.removeEventListener("mouseup", this.onMouseUp);
        this.canvas.removeEventListener("mousemove", this.onMouseMove);
    }
}

const cursorNormal = "url('../assets/icon-cursor-16.png'), auto";
const cursorNormalCanMove = "url('../assets/icon-cursor-click-16.png'), auto";
const cursorCrossHair = 'crosshair';
const cursorEditLine = "url('../assets/cursor-edit-line.cur'), auto";
const cursorEditArc = "url('../assets/cursor-edit-arc.cur'), auto";
const cursorEditBezier = "url('../assets/cursor-edit-bezier.cur'), auto";
const cursorEditSquare = "url('../assets/cursor-edit-square.cur'), auto";
const cursorEditCircle = "url('../assets/cursor-edit-circle.cur'), auto";

let editorState = 'pointer';
const canvas = document.getElementById('myCanvas');
const canvaContainer = document.getElementById('canvas-container');
const contextMenu = document.getElementById('contextMenu');

// Example action when clicking on a context menu item
document.getElementById('actionOne').addEventListener('click', (e) => {
    e.preventDefault();
    console.log('Action One clicked!');
});
// Hide the context menu when clicking elsewhere
window.addEventListener('click', (e) => {
    if (e.button !== 2) {  // Not a right-click
        contextMenu.style.display = 'none';
    }
});

document.getElementById("icon-pointer").addEventListener('click', function () {
    editorState = 'pointer';
});
document.getElementById("icon-selection").addEventListener('click', function () {
    editorState = 'selection';
});
document.getElementById("icon-line").addEventListener('click', function () {
    editorState = 'drawLine';
});
document.getElementById("icon-arc").addEventListener('click', function () {
    editorState = 'drawArc';
});
document.getElementById("icon-bezier").addEventListener('click', function () {
    editorState = 'drawBezier';
});
document.getElementById("icon-circle").addEventListener('click', function () {
    editorState = 'drawCircle';
});
document.getElementById("icon-square").addEventListener('click', function () {
    editorState = 'drawSquare';
});

const icons = document.querySelectorAll(".icon");
const tooltip = document.createElement("div");
tooltip.className = "tooltip";
document.body.appendChild(tooltip);

icons.forEach(icon => {
    icon.addEventListener('mouseover', function (e) {
        tooltip.textContent = e.target.getAttribute('data-tooltip');
        tooltip.style.display = 'block';
        tooltip.style.left = (e.pageX + 10) + 'px';  // 10 pixels to the right of the mouse pointer
        tooltip.style.top = (e.pageY + 10) + 'px';   // 10 pixels below the mouse pointer
    });
    icon.addEventListener('click', function () {
        if (icon.id !== 'icon-cog') {
            // First, remove 'icon-selected' from all icons
            icons.forEach(i => i.classList.remove("icon-selected"));
            // Then, add 'icon-selected' to the clicked icon
            icon.classList.add("icon-selected");
        }
    });
    icon.addEventListener('mouseout', function () {
        tooltip.style.display = 'none';
    });
});

canvas.width = 1500;  // minus left-panel width
canvas.height = 3000; // minus top-menu height
let workArea = new PlayingArea(window, canvas, canvaContainer);

