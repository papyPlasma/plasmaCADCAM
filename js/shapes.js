import {
    isPointOnPoint, isPointOnSegment, isPointOnArc, isPointOnBezier, isPointOnEllipse,
    findArcCenter, moveCenterEquidistant, findArcNewCenter, distanceBetweenPoints,
    snapToGrid, add, sub, zero
} from './math.js';

export const strokeSelected = getComputedStyle(document.documentElement).getPropertyValue('--canvas-stroke-selection').trim();
export const strokeDefault = getComputedStyle(document.documentElement).getPropertyValue('--canvas-stroke-default').trim();
export const strokeLight = getComputedStyle(document.documentElement).getPropertyValue('--canvas-stroke-light').trim();

class Shape {
    static counterId = 0;
    constructor(ctx, start, type = "generic") {
        this.ctx = ctx;
        this.offset = start;
        this.id = Shape.counterId;
        Shape.counterId += 1;
        this.e = { x: 10, y: 10 };
        this.se = 4;
        this.handles;
        this.type = type;
        // -2 = no selection, -1 = shape selected, >=0 handle idx selected
        this.selection = -2;
    }
    rX(p) { return this.offset.x + p.x; }
    rY(p) { return this.offset.y + p.y; }
    rXY(p) { return { x: this.rX(p), y: this.rY(p) } }

    move(delta) {
        this.offset = add(this.offset, delta);
    }

    removeSelection() {
        this.selection = -2;
    }

    hasSelection() {
        return this.selection > -2;
    }

    selectNoFill(pos) {
        this.ctx.fillStyle = "white";
        this.select(pos);
    }
    selectFill(pos) {
        this.ctx.fillStyle = "black";
        this.select(pos)
    }
    select(pos) {
        this.ctx.beginPath();
        this.ctx.rect(pos.x - this.se, pos.y - this.se, 2 * this.se, 2 * this.se);
        this.ctx.fill();
        this.ctx.stroke();
    }
}

export class Line extends Shape {
    constructor(ctx, start, end) {
        super(ctx, start, "lin");
        this.handles = [zero(), { x: end.x - start.x, y: end.y - start.y }];
        this.selection = 1; // end selected
    }
    draw() {
        this.ctx.strokeStyle = strokeDefault;
        const start = this.handles[0];
        const end = this.handles[1];
        let p = new Path2D();
        p.moveTo(this.rX(start), this.rY(start));
        p.lineTo(this.rX(end), this.rY(end));
        this.ctx.stroke(p);
        switch (this.selection) {
            case -1:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(end));
                break;
            case 0:
                this.selectFill(this.rXY(start));
                this.selectNoFill(this.rXY(end));
                break;
            case 1:
                this.selectNoFill(this.rXY(start));
                this.selectFill(this.rXY(end));
                break;
            default:
                break;
        }
    }
    setSelection(pos) {
        const start = this.handles[0];
        const end = this.handles[1];

        if (isPointOnPoint(pos, this.rXY(start), 5))
            this.selection = 0;
        else if (isPointOnPoint(pos, this.rXY(end), 5))
            this.selection = 1;
        else if (isPointOnSegment(pos, this.rXY(start), this.rXY(end), 5))
            this.selection = -1;
        else
            this.selection = -2;
        if (this.selection > -2)
            return true;
        else
            return false;
    }
    modify(dcursor, delta) {
        if (this.selection === -1)
            this.move(delta);
        else {
            this.handles[this.selection] = add(this.handles[this.selection], delta);
        }
    }
    snap(spacing) {
        if (this.selection == -1)
            this.offset = snapToGrid(this.offset, spacing);
        else
            this.handles[this.selection] = snapToGrid(this.handles[this.selection], spacing);
    }
    valid() {
        return (this.handles[0].x !== this.handles[1].x) || (this.handles[0].y !== this.handles[1].y);
    }
    getBoundingBox() {
        return {
            bl: this.offset,
            tr: add(this.offset, this.handles[1]),
        };
    }
}

export class Arc extends Shape {
    constructor(ctx, start, end) {
        super(ctx, start, "arc");
        this.init = true;
        this.radius = 50;
        this.handles = [zero(), { x: end.x - start.x, y: end.y - start.y },
        findArcCenter(zero(), { x: end.x - start.x, y: end.y - start.y }, this.radius)];
        this.selection = 1; // end selected
    }
    draw() {
        this.ctx.strokeStyle = strokeDefault;
        let p = new Path2D();
        const start = this.handles[0];
        const end = this.handles[1];
        const center = this.handles[2];
        const startAngle = Math.atan2(start.y - center.y, start.x - center.x);
        const endAngle = Math.atan2(end.y - center.y, end.x - center.x);
        p.arc(this.rX(center), this.rY(center), this.radius, startAngle, endAngle);
        this.ctx.stroke(p);
        if (this.selection > -2) {
            this.ctx.strokeStyle = strokeLight;
            this.ctx.setLineDash([3, 3]);
            this.ctx.beginPath();
            this.ctx.moveTo(this.rX(start), this.rY(start));
            this.ctx.lineTo(this.rX(center), this.rY(center));
            this.ctx.moveTo(this.rX(end), this.rY(end));
            this.ctx.lineTo(this.rX(center), this.rY(center));
            this.ctx.stroke();
            this.ctx.setLineDash([]);
            this.ctx.strokeStyle = strokeDefault;
        }
        switch (this.selection) {
            case -1:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(end));
                this.selectNoFill(this.rXY(center));
                break;
            case 0:
                this.selectFill(this.rXY(start));
                this.selectNoFill(this.rXY(end));
                this.selectNoFill(this.rXY(center));
                break;
            case 1:
                this.selectNoFill(this.rXY(start));
                this.selectFill(this.rXY(end));
                this.selectNoFill(this.rXY(center));
                break;
            case 2:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(end));
                this.selectFill(this.rXY(center));
                break;
            default:
                break;
        }
    }
    setSelection(pos) {
        const start = this.handles[0];
        const end = this.handles[1];
        const center = this.handles[2];
        const startAngle = Math.atan2(start.y - center.y, start.x - center.x);
        const endAngle = Math.atan2(end.y - center.y, end.x - center.x);
        if (isPointOnPoint(pos, this.rXY(start), 5))
            this.selection = 0;
        else if (isPointOnPoint(pos, this.rXY(end), 5))
            this.selection = 1;
        else if (isPointOnPoint(pos, this.rXY(center), 5)) {
            if (this.selection > -2)
                this.selection = 2;
            else
                this.selection = -2;
        }
        else if (isPointOnArc(pos, this.rXY(center), this.radius, startAngle, endAngle, 5))
            this.selection = -1;
        else
            this.selection = -2;
        if (this.selection > -2)
            return true;
        else
            return false;
    }
    modify(cursor, delta) {
        if (this.selection === -1)
            this.move(delta);
        else {
            if (this.selection == 0) {
                const tmp = this.handles[2];
                this.handles[0] = add(this.handles[0], delta);
                this.handles[2] = findArcNewCenter(this.handles[2], this.handles[0],
                    this.handles[1], this.radius);
                if (Number.isNaN(this.handles[2].x) || Number.isNaN(this.handles[2].x)) {
                    this.handles[0] = sub(this.handles[0], delta);
                    this.handles[2] = tmp;
                }
            } else if (this.selection == 1) {
                const tmp = this.handles[2];
                this.handles[1] = add(this.handles[1], delta);

                if (this.init) {
                    // Move also the center
                    this.radius = distanceBetweenPoints(this.handles[0], this.handles[1]);
                    this.handles[2] = findArcCenter(this.handles[0], this.handles[1], this.radius);
                } else {
                    this.handles[2] = findArcNewCenter(this.handles[2], this.handles[0],
                        this.handles[1], this.radius);

                    if (Number.isNaN(this.handles[2].x) || Number.isNaN(this.handles[2].x)) {
                        this.handles[1] = sub(this.handles[1], delta);
                        this.handles[2] = tmp;
                    }
                }

            } else {
                this.handles[2] = moveCenterEquidistant(this.handles[0], this.handles[1], this.handles[2], delta);
                this.radius = distanceBetweenPoints(this.handles[0], this.handles[2]);
            }
        }
    }
    snap(spacing) {
        if (this.selection == -1)
            this.offset = snapToGrid(this.offset, spacing);
        else if (this.selection == 0 || this.selection == 1) {
            this.handles[this.selection] = snapToGrid(this.handles[this.selection], spacing);
            this.handles[2] = findArcNewCenter(this.handles[2], this.handles[0], this.handles[1], this.radius);
            if (Number.isNaN(this.handles[2].x) || Number.isNaN(this.handles[2].x)) {
                this.radius += spacing; // Hack
                this.handles[2] = findArcNewCenter(this.handles[2], this.handles[0], this.handles[1], this.radius);
            }
        }
        this.init = false;
    }
    valid() {
        return (this.handles[0].x !== this.handles[1].x) || (this.handles[0].y !== this.handles[1].y);
    }
    getBoundingBox() {
        return {
            bl: this.offset,
            tr: add(this.offset, this.handles[1]),
        };
    }
}

export class Bezier extends Shape {
    constructor(ctx, start, ctrl1, ctrl2, end) {
        super(ctx, start, "bez");
        this.init = true;
        this.handles = [zero(), { x: ctrl1.x - start.x, y: ctrl1.y - start.y },
        { x: ctrl2.x - start.x, y: ctrl2.y - start.y }, { x: end.x - start.x, y: end.y - start.y }]
        this.selection = 3; // end selected
    }
    draw() {
        this.ctx.strokeStyle = strokeDefault;
        let p = new Path2D();
        const start = this.handles[0];
        const ctrl1 = this.handles[1];
        const ctrl2 = this.handles[2];
        const end = this.handles[3];
        p.moveTo(this.rX(start), this.rY(start));
        p.bezierCurveTo(this.rX(ctrl1), this.rY(ctrl1),
            this.rX(ctrl2), this.rY(ctrl2),
            this.rX(end), this.rY(end)
        );
        this.ctx.stroke(p);
        if (this.selection > -2) {
            this.ctx.strokeStyle = strokeLight;
            this.ctx.setLineDash([3, 3]);
            this.ctx.beginPath();
            this.ctx.moveTo(this.rX(start), this.rY(start));
            this.ctx.lineTo(this.rX(ctrl1), this.rY(ctrl1));
            this.ctx.moveTo(this.rX(end), this.rY(end));
            this.ctx.lineTo(this.rX(ctrl2), this.rY(ctrl2));
            this.ctx.stroke();
            this.ctx.setLineDash([]);
            this.ctx.strokeStyle = strokeDefault;
        }
        switch (this.selection) {
            case -1:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(ctrl1));
                this.selectNoFill(this.rXY(ctrl2));
                this.selectNoFill(this.rXY(end));
                break;
            case 0:
                this.selectFill(this.rXY(start));
                this.selectNoFill(this.rXY(ctrl1));
                this.selectNoFill(this.rXY(ctrl2));
                this.selectNoFill(this.rXY(end));
                break;
            case 1:
                this.selectNoFill(this.rXY(start));
                this.selectFill(this.rXY(ctrl1));
                this.selectNoFill(this.rXY(ctrl2));
                this.selectNoFill(this.rXY(end));
                break;
            case 2:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(ctrl1));
                this.selectFill(this.rXY(ctrl2));
                this.selectNoFill(this.rXY(end));
                break;
            case 3:
                this.selectNoFill(this.rXY(start));
                this.selectNoFill(this.rXY(ctrl1));
                this.selectNoFill(this.rXY(ctrl2));
                this.selectFill(this.rXY(end));
                break;
            default:
                break;
        }
    }
    setSelection(pos) {
        const start = this.handles[0];
        const ctrl1 = this.handles[1];
        const ctrl2 = this.handles[2];
        const end = this.handles[3];
        if (isPointOnPoint(pos, this.rXY(start), 5))
            this.selection = 0;
        else if (isPointOnPoint(pos, this.rXY(ctrl1), 5)) {
            if (this.selection > -2)
                this.selection = 1;
            else
                this.selection = -2;
        }
        else if (isPointOnPoint(pos, this.rXY(ctrl2), 5)) {
            if (this.selection > -2)
                this.selection = 2;
            else
                this.selection = -2;
        }
        else if (isPointOnPoint(pos, this.rXY(end), 5))
            this.selection = 3;
        else if (isPointOnBezier(pos,
            this.rXY(start),
            this.rXY(ctrl1),
            this.rXY(ctrl2),
            this.rXY(end), 10))
            this.selection = -1;
        else
            this.selection = -2;
        if (this.selection > -2)
            return true;
        else
            return false;
    }
    modify(cursor, delta) {
        if (this.selection === -1)
            this.move(delta);
        else {
            this.handles[this.selection] = add(this.handles[this.selection], delta);
            if (this.init)
                if (this.selection === 3) {
                    this.handles[1].x = this.handles[3].x / 3;
                    this.handles[1].y = this.handles[3].y / 3 + 30;
                    this.handles[2].x = 2 * this.handles[3].x / 3;
                    this.handles[2].y = 2 * this.handles[3].y / 3 - 30;
                }
        }
    }
    snap(spacing) {
        if (this.selection == -1)
            this.offset = snapToGrid(this.offset, spacing);
        else
            this.handles[this.selection] = snapToGrid(this.handles[this.selection], spacing);
        this.init = false;
    }
    valid() {
        return (this.handles[0].x !== this.handles[3].x) || (this.handles[0].y !== this.handles[3].y);
    }
    getBoundingBox() {
        return {
            bl: this.offset,
            tr: add(this.offset, this.handles[3]),
        };
    }
}

export class Square extends Shape {
    constructor(ctx, start, edge) {
        super(ctx, start, "squ");
        this.edge = edge;
        this.handles = [zero(), { x: 0, y: this.edge.y },
        { x: this.edge.x, y: 0 }, this.edge,
        { x: 0, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: this.edge.y },
        { x: this.edge.x, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: 0 },
        ];
        this.selection = 3; // end selected
    }
    draw() {
        this.ctx.strokeStyle = strokeDefault;
        const bl = this.handles[0];
        const tl = this.handles[1];
        const br = this.handles[2];
        const tr = this.handles[3];
        const ml = this.handles[4];
        const mt = this.handles[5];
        const mr = this.handles[6];
        const mb = this.handles[7];
        let p = new Path2D();
        p.moveTo(this.rX(bl), this.rY(bl));
        p.lineTo(this.rX(tl), this.rY(tl));
        p.lineTo(this.rX(tr), this.rY(tr));
        p.lineTo(this.rX(br), this.rY(br));
        p.lineTo(this.rX(bl), this.rY(bl));
        this.ctx.stroke(p);
        // Draw dotted line if square
        if (this.selection > -2) {
            if (Math.abs(Math.abs(this.edge.x) - Math.abs(this.edge.y)) < 2) {
                this.ctx.strokeStyle = strokeLight;
                this.ctx.setLineDash([3, 3]);
                this.ctx.beginPath();
                this.ctx.moveTo(this.rX(bl), this.rY(bl));
                this.ctx.lineTo(this.rX(tr), this.rY(tr));
                this.ctx.moveTo(this.rX(tl), this.rY(tl));
                this.ctx.lineTo(this.rX(br), this.rY(br));
                this.ctx.stroke();
                this.ctx.setLineDash([]);
                this.ctx.strokeStyle = strokeDefault;
            }
        }
        switch (this.selection) {
            case -1:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 0:
                this.selectFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 1:
                this.selectNoFill(this.rXY(bl));
                this.selectFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 2:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 3:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 4:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 5:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 6:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectFill(this.rXY(mr));
                this.selectNoFill(this.rXY(mb));
                break;
            case 7:
                this.selectNoFill(this.rXY(bl));
                this.selectNoFill(this.rXY(tl));
                this.selectNoFill(this.rXY(br));
                this.selectNoFill(this.rXY(tr));
                this.selectNoFill(this.rXY(ml));
                this.selectNoFill(this.rXY(mt));
                this.selectNoFill(this.rXY(mr));
                this.selectFill(this.rXY(mb));
                break;
            default:
                break;
        }
    }
    setSelection(pos) {
        const bl = this.handles[0];
        const tl = this.handles[1];
        const br = this.handles[2];
        const tr = this.handles[3];
        const ml = this.handles[4];
        const mt = this.handles[5];
        const mr = this.handles[6];
        const mb = this.handles[7];

        if (isPointOnPoint(pos, this.rXY(bl), 5))
            this.selection = 0;
        else if (isPointOnPoint(pos, this.rXY(tl), 5))
            this.selection = 1;
        else if (isPointOnPoint(pos, this.rXY(br), 5))
            this.selection = 2;
        else if (isPointOnPoint(pos, this.rXY(tr), 5))
            this.selection = 3;
        else if (isPointOnPoint(pos, this.rXY(ml), 5))
            this.selection = 4;
        else if (isPointOnPoint(pos, this.rXY(mt), 5))
            this.selection = 5;
        else if (isPointOnPoint(pos, this.rXY(mr), 5))
            this.selection = 6;
        else if (isPointOnPoint(pos, this.rXY(mb), 5))
            this.selection = 7;
        else
            if (isPointOnSegment(pos, this.rXY(bl), this.rXY(tl), 5) ||
                isPointOnSegment(pos, this.rXY(tl), this.rXY(tr), 5) ||
                isPointOnSegment(pos, this.rXY(tr), this.rXY(br), 5) ||
                isPointOnSegment(pos, this.rXY(br), this.rXY(bl), 5))
                this.selection = -1;
            else
                this.selection = -2;
        if (this.selection > -2)
            return true;
        else
            return false;
    }
    modify(cursor, delta) {
        const bl = this.handles[0];
        const tl = this.handles[1];
        const br = this.handles[2];
        const tr = this.handles[3];
        const ml = this.handles[4];
        const mt = this.handles[5];
        const mr = this.handles[6];
        const mb = this.handles[7];
        if (this.selection === -1)
            this.move(delta);
        else {
            // On vertices
            if (this.selection === 0) {
                this.offset.x += delta.x;
                this.offset.y += delta.y;
                this.edge.x = br.x - bl.x - delta.x;
                this.edge.y = tr.y - br.y - delta.y;
            }
            else if (this.selection === 1) {
                this.offset.x += delta.x;
                this.edge.x = br.x - bl.x - delta.x;
                this.edge.y = tr.y - br.y + delta.y;
            } else if (this.selection === 2) {
                this.offset.y += delta.y;
                this.edge.x = br.x - bl.x + delta.x;
                this.edge.y = tr.y - br.y - delta.y;
            }
            else if (this.selection === 3) {
                this.edge.x = br.x - bl.x + delta.x;
                this.edge.y = tr.y - br.y + delta.y;
            }
            // On edges
            else if (this.selection === 4) {
                this.offset.x += delta.x;
                this.edge.x = br.x - bl.x - delta.x;
            }
            else if (this.selection === 5) {
                //
                this.edge.y = tr.y - br.y + delta.y;;
            } else if (this.selection === 6) {
                //
                this.edge.x = br.x - bl.x + delta.x;
            }
            else if (this.selection === 7) {
                this.offset.y += delta.y;
                this.edge.y = tr.y - br.y - delta.y;
            }

            this.handles = [zero(), { x: 0, y: this.edge.y },
            { x: this.edge.x, y: 0 }, this.edge,
            { x: 0, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: this.edge.y },
            { x: this.edge.x, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: 0 },
            ];
        }
    }
    snap(spacing) {
        if (this.selection == -1)
            this.offset = snapToGrid(this.offset, spacing);
        else {
            if (this.selection === 0) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 1) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 2) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 3) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 4) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 5) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 6) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            else if (this.selection === 7) {
                this.offset = snapToGrid(this.offset, spacing);
                this.edge = snapToGrid(this.edge, spacing);
            }
            // No degenerate
            if (this.edge.x === 0) {
                this.edge.x = spacing;
            }
            if (this.edge.y === 0) {
                this.edge.y = spacing;
            }


            this.handles = [zero(), { x: 0, y: this.edge.y },
            { x: this.edge.x, y: 0 }, this.edge,
            { x: 0, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: this.edge.y },
            { x: this.edge.x, y: this.edge.y / 2 }, { x: this.edge.x / 2, y: 0 },
            ];
        }
    }
    valid() {
        return (this.handles[0].x !== this.handles[3].x) && (this.handles[0].y !== this.handles[3].y);
    }
    getBoundingBox() {
        return {
            bl: this.offset,
            tr: add(this.offset, this.handles[3]),
        };
    }
}

export class Circle extends Shape {
    constructor(ctx, start, radius) {
        super(ctx, start, "squ");
        this.radius = radius;
        this.handles = [zero(), { x: -this.radius.x, y: 0 },
        { x: 0, y: this.radius.y }, { x: this.radius.x, y: 0 },
        { x: 0, y: -this.radius.y }, { x: this.radius.x, y: this.radius.y }
        ];
        this.selection = 5; // r, end selected
    }
    draw() {
        this.ctx.strokeStyle = strokeDefault;
        const center = this.handles[0];
        const l = this.handles[1];
        const t = this.handles[2];
        const r = this.handles[3];
        const b = this.handles[4];
        const tr = this.handles[5];
        let p = new Path2D();
        p.ellipse(this.rX(center), this.rY(center), this.radius.x, this.radius.y, 0, 0, 2 * Math.PI);
        this.ctx.stroke(p);
        switch (this.selection) {
            case -1:
                this.selectNoFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 0:
                this.selectFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 1:
                this.selectNoFill(this.rXY(center));
                this.selectFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 2:
                this.selectNoFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 3:
                this.selectNoFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 4:
                this.selectNoFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectFill(this.rXY(b));
                this.selectNoFill(this.rXY(tr));
                break;
            case 5:
                this.selectNoFill(this.rXY(center));
                this.selectNoFill(this.rXY(l));
                this.selectNoFill(this.rXY(t));
                this.selectNoFill(this.rXY(r));
                this.selectNoFill(this.rXY(b));
                this.selectFill(this.rXY(tr));
                break;
            default:
                break;
        }
    }
    setSelection(pos) {
        const center = this.handles[0];
        const l = this.handles[1];
        const t = this.handles[2];
        const r = this.handles[3];
        const b = this.handles[4];
        const tr = this.handles[5];
        if (isPointOnPoint(pos, this.rXY(center), 5))
            this.selection = 0;
        else if (isPointOnPoint(pos, this.rXY(l), 5))
            this.selection = 1;
        else if (isPointOnPoint(pos, this.rXY(t), 5))
            this.selection = 2;
        else if (isPointOnPoint(pos, this.rXY(r), 5))
            this.selection = 3;
        else if (isPointOnPoint(pos, this.rXY(b), 5))
            this.selection = 4;
        else if (isPointOnPoint(pos, this.rXY(tr), 5))
            this.selection = 5;
        else
            if (isPointOnEllipse(pos, this.rXY(center), this.radius, 0.5))
                this.selection = -1;
            else
                this.selection = -2;
        if (this.selection > -2)
            return true;
        else
            return false;
    }
    modify(cursor, delta) {
        const center = this.handles[0];
        const l = this.handles[1];
        const t = this.handles[2];
        const r = this.handles[3];
        const b = this.handles[4];
        const tr = this.handles[5];
        if (this.selection === -1 || this.selection === 0)
            this.move(delta);
        else {
            if (this.selection === 1) {
                this.radius.x -= delta.x;
            } else if (this.selection === 2) {
                this.radius.y += delta.y;
            }
            else if (this.selection === 3) {
                this.radius.x += delta.x;
            }
            else if (this.selection === 4) {
                this.radius.y -= delta.y;
            } else if (this.selection === 5) {
                this.radius.x = Math.abs(cursor.x - this.rX(center));
                this.radius.y = Math.abs(cursor.y - this.rY(center));
            }
            if (this.radius.x < 0)
                this.radius.x = -this.radius.x;
            if (this.radius.y < 0)
                this.radius.y = -this.radius.y;
            this.handles = [zero(), { x: -this.radius.x, y: 0 },
            { x: 0, y: this.radius.y }, { x: this.radius.x, y: 0 },
            { x: 0, y: -this.radius.y }, { x: this.radius.x, y: this.radius.y }
            ];
        }
    }
    snap(spacing) {
        if (this.selection == -1)
            this.offset = snapToGrid(this.offset, spacing);
        else {
            this.offset = snapToGrid(this.offset, spacing);
            this.radius = snapToGrid(this.radius, spacing);
            // No degenerate
            if (this.radius.x === 0) {
                this.radius.x = spacing;
            }
            if (this.radius.y === 0) {
                this.radius.y = spacing;
            }
            this.handles = [zero(), { x: -this.radius.x, y: 0 },
            { x: 0, y: this.radius.y }, { x: this.radius.x, y: 0 },
            { x: 0, y: -this.radius.y }, { x: this.radius.x, y: this.radius.y }
            ];
        }
    }
    valid() {
        return (this.radius.x > 0 && this.radius.y > 0);
    }
    getBoundingBox() {
        return {
            bl: sub(this.offset, { x: this.radius, y: this.radius }),
            tr: add(this.offset, { x: this.radius, y: this.radius }),
        };
    }
}