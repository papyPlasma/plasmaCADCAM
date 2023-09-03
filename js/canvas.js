import { Line, Arc, Bezier, Square, Circle, strokeLight, strokeDefault } from './shapes.js';
import { snapToGrid, zero, isBoxInside } from './math.js';

class PlayingArea {
    constructor(window, canvasContainer) {
        this.window = window;
        this.container = canvasContainer;
        this.canvas = document.getElementById("myCanvas");
        this.ctx = this.canvas.getContext("2d");

        // Canvas parameters
        this.gridSpacing = 10;  // Change to desired spacing
        this.margin = 20;
        this.axisColor = "#000";
        this.gridColor = "#eee";

        // Zoom 
        this.mouseIsDown = false;
        this.prevX = 0;
        this.prevY = 0;
        this.prevCursor = zero();
        this.scale = 1;
        this.offsetX = 0;
        this.offsetY = 0;

        // Event listeners
        this.window.addEventListener('resize', this.onResizeWindow.bind(this));
        this.canvas.addEventListener("wheel", this.onWheel.bind(this), { passive: true });
        this.canvas.addEventListener("mousedown", this.onMouseDown.bind(this));
        this.canvas.addEventListener("mouseup", this.onMouseUp.bind(this));
        this.canvas.addEventListener("mousemove", this.onMouseMove.bind(this));
        this.canvas.addEventListener("keydown", this.handleKeyDown.bind(this));

        this.shapes = [];
        this.currentShape = undefined;
        this.selectionArea = {
            bl: { x: 0, y: 0 },
            tr: { x: 0, y: 0 },
        };

        // Few examples
        let line = new Line(this.ctx, { x: 500, y: 400 }, { x: 550, y: 500 });
        this.shapes.push(line);

        let arc = new Arc(this.ctx, { x: 400, y: 400 }, { x: 450, y: 430 });
        this.shapes.push(arc);

        let bezier = new Bezier(this.ctx, zero(), { x: 100, y: 100 },
            { x: 200, y: 0 }, { x: 300, y: 100 });
        bezier.move({ x: 400, y: 100 });
        this.shapes.push(bezier);

        for (const shape of this.shapes) {
            shape.removeSelection();
        }

        this.onResizeWindow();
        this.canvas.focus();
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
        const relativeX = clickX / this.scale - this.margin;
        const relativeY = this.canvas.height - clickY / this.scale - this.margin;
        return { x: relativeX, y: relativeY };
    }

    onResizeWindow() {
        this.canvas.width = this.container.offsetWidth;
        this.canvas.height = this.container.offsetHeight;
        this.render();
    }

    onWheel(event) {
        const zoomFactor = 0.1;
        let newScale;

        // Get mouse position relative to the canvas
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = event.clientX - rect.left - this.offsetX;
        const mouseY = event.clientY - rect.top - this.offsetY;

        // Calculate cursor's position relative to the content (before zooming)
        const relativeX = mouseX / this.scale;
        const relativeY = mouseY / this.scale;

        if (event.deltaY < 0) {
            // Zoom in
            newScale = this.scale * (1 + zoomFactor);
        } else {
            // Zoom out
            newScale = this.scale / (1 + zoomFactor);

            // Prevent zooming out beyond 100%
            if (newScale < 1) {
                newScale = 1;
            }
        }

        // Adjust offsets to ensure cursor's relative position remains the same after zooming
        this.offsetX += mouseX - (relativeX * newScale);
        this.offsetY += mouseY - (relativeY * newScale);

        this.scale = newScale;

        // Clamp offsets after adjusting for zoom
        this.clampOffsets();

        this.render();
        //  event.preventDefault();
    }
    goToselectionMode() {
        let pointerIcon = document.getElementById('icon-selection');
        icons.forEach(i => i.classList.remove("icon-selected"));
        pointerIcon.classList.add("icon-selected");
        editorState = 'selection';
        canvasCursor = cursorNormal;
        canvas.style.cursor = cursorNormal;
    }
    goToPointerMode() {
        let pointerIcon = document.getElementById('icon-pointer');
        icons.forEach(i => i.classList.remove("icon-selected"));
        pointerIcon.classList.add("icon-selected");
        editorState = 'pointer';
        canvasCursor = cursorNormal;
        canvas.style.cursor = cursorNormal;
    }
    onMouseDown(event) {
        let start;
        if (event.button === 0) {
            this.mouseIsDown = true;
            this.prevX = event.clientX;
            this.prevY = event.clientY;
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
                        console.log("add shape");
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
                                console.log(bb);
                                shape.selection = -1;
                            }
                        }
                    }
                    this.selectionArea = {
                        bl: { x: 0, y: 0 },
                        tr: { x: 0, y: 0 },
                    };
                    this.goToPointerMode();
                    break;

                default:
                    break;
            }
        }
        this.onMouseMove(event);
    }

    onMouseMove(event) {
        this.canvas.style.cursor = canvasCursor;
        const deltaAbsoluteX = event.clientX - this.prevX;
        const deltaAbsoluteY = event.clientY - this.prevY;
        let cursor = this.getRelativePositionCursor({ x: event.clientX, y: event.clientY });
        const deltaCursor = { x: cursor.x - this.prevCursor.x, y: cursor.y - this.prevCursor.y };
        if (this.mouseIsDown) {
            switch (editorState) {
                case 'pointer':
                    let somethingSelected = false;
                    for (const shape of this.shapes) {
                        if (shape.selection > -2) {
                            shape.modify(deltaCursor);
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
                        // Restrict the content from being dragged too far out of view.
                        this.clampOffsets();
                    }
                    break;

                case 'drawLine':
                case 'drawArc':
                case 'drawBezier':
                case 'drawSquare':
                case 'drawCircle':
                    this.currentShape.modify(deltaCursor);
                    break;

                case 'selection':
                    this.selectionArea.tr.x += deltaCursor.x;
                    this.selectionArea.tr.y += deltaCursor.y;
                    break;

                default:
                    this.canvas.style.cursor = canvasCursor;
                    break;
            }
        }
        // Update previous cursor position for next movement calculation
        this.prevX = event.clientX;
        this.prevY = event.clientY;
        this.prevCursor = cursor;
        this.render();
    }

    clampOffsets() {
        // Calculate the minimum permissible offsets.
        const minX = this.canvas.width - this.canvas.width * this.scale;
        const minY = this.canvas.height - this.canvas.height * this.scale;

        // Maximum permissible offsets are always 0 because 
        // the content should not drift away from the top-left corner.
        const maxX = 0;
        const maxY = 0;

        // Clamp offsets to ensure content stays within the canvas boundaries.
        this.offsetX = Math.min(Math.max(this.offsetX, minX), maxX);
        this.offsetY = Math.min(Math.max(this.offsetY, minY), maxY);
    }

    render() {
        this.ctx.setTransform(1, 0, 0, 1, 0, 0);
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.setTransform(this.scale, 0, 0, this.scale, this.offsetX, this.offsetY);
        // Set the origin to the bottom left
        this.ctx.translate(this.margin, this.canvas.height - this.margin);  // Translate by the canvas height
        this.ctx.scale(1, -1);  // Flip vertically

        this.drawAll();
    }

    drawAll() {
        this.drawGrid();
        this.drawOrigin();
        this.drawContent();
    }

    drawGrid() {
        // Draw the margins
        this.ctx.fillStyle = "#ddd";
        this.ctx.fillRect(-this.margin, -this.margin, this.canvas.width, this.margin);
        this.ctx.fillRect(-this.margin, -this.margin, this.margin, this.canvas.height);
        this.ctx.fillRect(-this.margin, this.canvas.height - 2 * this.margin, this.canvas.width, this.margin);
        this.ctx.fillRect(this.canvas.width - 2 * this.margin, 0, this.margin, this.canvas.height);

        this.ctx.lineWidth = 1;
        this.ctx.strokeStyle = this.gridColor;
        // Vertical grid lines
        for (let x = 0; x <= this.canvas.width - 2 * this.margin; x += this.gridSpacing) {
            this.ctx.beginPath();
            this.ctx.moveTo(x, 0);
            this.ctx.lineTo(x, this.canvas.height - 2 * this.margin);
            this.ctx.stroke();
        }

        // Horizontal grid lines
        for (let y = 0; y <= this.canvas.height - 2 * this.margin; y += this.gridSpacing) {
            this.ctx.beginPath();
            this.ctx.moveTo(0, y);
            this.ctx.lineTo(this.canvas.width - 2 * this.margin, y);
            this.ctx.stroke();
        }

        // Axis
        this.ctx.strokeStyle = 'blue';
        this.ctx.lineWidth = 2;

        this.ctx.beginPath();
        this.ctx.moveTo(0, 0);
        this.ctx.lineTo(this.canvas.width - 2 * this.margin, 0);
        this.ctx.stroke();

        this.ctx.beginPath();
        this.ctx.moveTo(0, 0);
        this.ctx.lineTo(0, this.canvas.height - 2 * this.margin);
        this.ctx.stroke();
    }

    drawOrigin() {
        const circleRadius = 10; // Adjust as needed
        const crossLength = 15;  // Adjust as needed

        // Draw circle
        this.ctx.beginPath();
        this.ctx.arc(0, 0, circleRadius, 0, 2 * Math.PI);
        this.ctx.fillStyle = "#000"; // Black color for the circle
        this.ctx.fill();
        this.ctx.closePath();

        // Draw the rotated cross
        this.ctx.strokeStyle = "#FFF"; // White color for the cross to contrast with the black circle
        this.ctx.lineWidth = 2;        // Adjust as needed

        this.ctx.save();
        // Rotate by 45 degrees
        this.ctx.rotate(Math.PI / 4);

        // Vertical line of the cross
        this.ctx.beginPath();
        this.ctx.moveTo(0, -crossLength / 2);
        this.ctx.lineTo(0, crossLength / 2);
        this.ctx.stroke();

        // Horizontal line of the cross
        this.ctx.beginPath();
        this.ctx.moveTo(-crossLength / 2, 0);
        this.ctx.lineTo(crossLength / 2, 0);
        this.ctx.stroke();

        this.ctx.restore();
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

    // Destructor
    destroy() {
        this.window.removeEventListener('resize', this.onResizeWindow);
        this.canvas.removeEventListener("wheel", this.onWheel);
        this.canvas.removeEventListener("mousedown", this.onMouseDown);
        this.window.removeEventListener("mouseup", this.onMouseUp);
        this.canvas.removeEventListener("mousemove", this.onMouseMove);
    }
}

let editorState = 'pointer';
let _workArea = new PlayingArea(window, document.getElementById("canvas-container"));

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
        // First, remove 'icon-selected' from all icons
        icons.forEach(i => i.classList.remove("icon-selected"));

        // Then, add 'icon-selected' to the clicked icon
        icon.classList.add("icon-selected");
    });
    icon.addEventListener('mouseout', function () {
        tooltip.style.display = 'none';
    });
});

let canvasCursor;
const cursorNormal = "url('../assets/icon-cursor-16.png'), auto";
const cursorNormalCanMove = "url('../assets/icon-cursor-click-16.png'), auto";
const cursorCrossHair = 'crosshair';
const cursorEditLine = "url('../assets/cursor-edit-line.cur'), auto";
const cursorEditArc = "url('../assets/cursor-edit-arc.cur'), auto";
const cursorEditBezier = "url('../assets/cursor-edit-bezier.cur'), auto";
const cursorEditSquare = "url('../assets/cursor-edit-square.cur'), auto";
const cursorEditCircle = "url('../assets/cursor-edit-circle.cur'), auto";

const canvas = document.getElementById('myCanvas');
const contextMenu = document.getElementById('contextMenu');
canvas.addEventListener('mouseenter', function () {
    switch (editorState) {
        case 'pointer':
            canvasCursor = cursorNormal;
            break;
        case 'selection':
            canvasCursor = cursorCrossHair;
            break;
        case 'drawLine':
            canvasCursor = cursorEditLine;
            break;
        case 'drawArc':
            canvasCursor = cursorEditArc;
            break;
        case 'drawBezier':
            canvasCursor = cursorEditBezier;
            break;
        case 'drawSquare':
            canvasCursor = cursorEditSquare;
            break;
        case 'drawCircle':
            canvasCursor = cursorEditCircle;
            break;
        default:
            canvasCursor = cursorCrossHair;
            break;
    }
});

canvas.addEventListener('mouseleave', function () {
    canvasCursor = 'default';  // Reset to default cursor
});

canvas.addEventListener('contextmenu', (e) => {
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

// Hide the context menu when clicking elsewhere
window.addEventListener('click', (e) => {
    if (e.button !== 2) {  // Not a right-click
        contextMenu.style.display = 'none';
    }
});

// Example action when clicking on a context menu item
document.getElementById('actionOne').addEventListener('click', (e) => {
    e.preventDefault();
    console.log('Action One clicked!');
});

