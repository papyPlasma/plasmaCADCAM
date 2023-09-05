// Thanks ChatGPT4 :D

export function isPointOnPoint(p, a, e) {
    const dx = Math.abs(p.x - a.x);
    const dy = Math.abs(p.y - a.y);

    return dx < e && dy < e;
}

export function isPointOnSegment(p, a, b, e) {
    function distanceFromLine(p, a, b) {
        const numerator = Math.abs((b.y - a.y) * p.x - (b.x - a.x) * p.y + b.x * a.y - b.y * a.x);
        const denominator = Math.sqrt((b.y - a.y) ** 2 + (b.x - a.x) ** 2);
        return numerator / denominator;
    }
    function isBetween(a, b, p) {
        const dotProduct = (p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y);
        if (dotProduct < 0) return false;

        const squaredLengthBA = (b.x - a.x) ** 2 + (b.y - a.y) ** 2;
        if (dotProduct > squaredLengthBA) return false;

        return true;
    }
    if (distanceFromLine(p, a, b) > e) {
        return false;
    }
    return isBetween(a, b, p);
}

export function isPointOnArc(p, c, r, startAngle, endAngle, e) {
    function normalizeAngle(angle) {
        while (angle < 0) {
            angle += 2 * Math.PI;
        }
        while (angle >= 2 * Math.PI) {
            angle -= 2 * Math.PI;
        }
        return angle;
    }

    let distanceToCenter = Math.sqrt((p.x - c.x) ** 2 + (p.y - c.y) ** 2);
    let theta = Math.atan2(p.y - c.y, p.x - c.x);

    theta = normalizeAngle(theta);
    startAngle = normalizeAngle(startAngle);
    endAngle = normalizeAngle(endAngle);

    let lowerBound = r - e;
    let upperBound = r + e;

    if (startAngle <= endAngle) {
        return (startAngle <= theta && theta <= endAngle) && (lowerBound <= distanceToCenter && distanceToCenter <= upperBound);
    } else {
        // The arc wraps around 2π
        return (theta >= startAngle || theta <= endAngle) && (lowerBound <= distanceToCenter && distanceToCenter <= upperBound);
    }
}

export function isPointOnBezier(P, P0, P1, P2, P3, epsilon) {
    let tMin = 0;
    let tMax = 1;
    let minDist = Infinity;

    for (let i = 0; i < 100; i++) { // max iterations can be adjusted
        let tMid = (tMin + tMax) / 2;

        let Bt = {
            x: Math.pow(1 - tMid, 3) * P0.x + 3 * Math.pow(1 - tMid, 2) * tMid * P1.x + 3 * (1 - tMid) * Math.pow(tMid, 2) * P2.x + Math.pow(tMid, 3) * P3.x,
            y: Math.pow(1 - tMid, 3) * P0.y + 3 * Math.pow(1 - tMid, 2) * tMid * P1.y + 3 * (1 - tMid) * Math.pow(tMid, 2) * P2.y + Math.pow(tMid, 3) * P3.y
        };

        let dist = Math.sqrt(Math.pow(Bt.x - P.x, 2) + Math.pow(Bt.y - P.y, 2));

        if (dist < minDist) {
            minDist = dist;
        }

        if (dist < epsilon) {
            return true;  // We found a sufficiently close point
        }

        // Using gradient to decide the next tMid for the next iteration.
        let gradient = (Bt.x - P.x) * (P3.x - P0.x) + (Bt.y - P.y) * (P3.y - P0.y);

        if (gradient > 0) {
            tMax = tMid;
        } else {
            tMin = tMid;
        }
    }
    return minDist <= epsilon;
}

export function isPointOnEllipse(p, c, r, e) {
    const value = (p.x - c.x) ** 2 / r.x ** 2 + (p.y - c.y) ** 2 / r.y ** 2;
    return value < 1 + e && value > 1 - e;
}

export function findArcCenter(startPoint, endPoint, r) {
    // Find the midpoint of AB
    let mx = (startPoint.x + endPoint.x) / 2;
    let my = (startPoint.y + endPoint.y) / 2;

    // Direction vector of AB
    let dx = endPoint.x - startPoint.x;
    let dy = endPoint.y - startPoint.y;

    // Find the magnitude (length) of this direction
    let mag = Math.sqrt(dx * dx + dy * dy);

    // Normalize the direction vector
    dx /= mag;
    dy /= mag;

    // Find the perpendicular direction
    let px = -dy;
    let py = dx;

    // Calculate the distance to move from the midpoint M to find the center C
    let dist = Math.sqrt(r * r - (mag / 2) * (mag / 2));

    let cx = mx + dist * px;
    let cy = my + dist * py;

    return { x: cx, y: cy };
}

export function findArcNewCenter(originalCenter, start, end, r) {
    // Get the two possible centers
    const possibleCenters = findPossibleCenters(start, end, r);

    // Calculate distances to the original center
    const dist1 = getDistance(originalCenter, possibleCenters[0]);
    const dist2 = getDistance(originalCenter, possibleCenters[1]);

    // Return the center that's closer to the original center
    return dist1 < dist2 ? possibleCenters[0] : possibleCenters[1];
}

function findPossibleCenters(start, end, r) {
    // Find the midpoint of AB
    let mx = (start.x + end.x) / 2;
    let my = (start.y + end.y) / 2;

    // Direction vector of AB
    let dx = end.x - start.x;
    let dy = end.y - start.y;

    // Find the magnitude (length) of this direction
    let mag = Math.sqrt(dx * dx + dy * dy);

    // Normalize the direction vector
    dx /= mag;
    dy /= mag;

    // Find the perpendicular direction
    let px = -dy;
    let py = dx;

    // Calculate the distance to move from the midpoint M to find the center C
    // Using the Pythagorean theorem, we get the distance as:
    let dist = Math.sqrt(r * r - (mag / 2) * (mag / 2));

    // Two possible centers
    let cx1 = mx + dist * px;
    let cy1 = my + dist * py;

    let cx2 = mx - dist * px;
    let cy2 = my - dist * py;

    return [{ x: cx1, y: cy1 }, { x: cx2, y: cy2 }];
}

export function moveCenterEquidistant(start, end, originalCenter, delta) {
    // Compute the direction of the segment (end - start)
    let dirX = end.x - start.x;
    let dirY = end.y - start.y;

    // Normalize this direction
    let length = Math.sqrt(dirX * dirX + dirY * dirY);
    dirX /= length;
    dirY /= length;

    // Compute the perpendicular direction
    let perpX = -dirY;
    let perpY = dirX;

    // Compute the midpoint of the segment
    let midX = (start.x + end.x) / 2;
    let midY = (start.y + end.y) / 2;

    // The movement in the direction of the perpendicular bisector 
    // is the dot product of the delta with the perpendicular direction
    let movement = delta.x * perpX + delta.y * perpY;

    // Move the center along the perpendicular direction by the computed amount
    let centerX = originalCenter.x + movement * perpX;
    let centerY = originalCenter.y + movement * perpY;

    return { x: centerX, y: centerY };
}

export function getDistance(point1, point2) {
    const dx = point1.x - point2.x;
    const dy = point1.y - point2.y;
    return Math.sqrt(dx * dx + dy * dy);
}

export function getAngle(pt1, pt2) {
    let a;
    if (pt1.x === pt2.x) {
        if (pt1.y < pt2.y)
            return Math.PI / 2;
        else
            return -Math.PI / 2;
    }
    if (pt1.y === pt2.y) {
        if (pt1.x < pt2.x)
            return 0;
        else
            return Math.PI;
    }
    a = Math.PI - Math.atan2(-(pt1.y - pt2.y), pt1.x - pt2.x);
    if (a > Math.PI)
        a = -(2 * Math.PI - a);
    return a;
}
export function subAngle(endAngle, startAngle) {
    let res = endAngle - startAngle;
    if (res > Math.PI)
        res -= Math.PI;
    if (res < -Math.PI)
        res += Math.PI;
    return res;
}

export function getMidPoint(point1, point2) {
    const x = (point1.x + point2.x) / 2;
    const y = (point1.y + point2.y) / 2;
    return { x, y };
}

export function getPerpendicularSegment(point1, point2) {
    const mid = getMidPoint(point1, point2);
    let dx = -(point2.y - point1.y) / 2;
    let dy = (point2.x - point1.x) / 2;
    return { p1: { x: mid.x - dx, y: mid.y - dy }, p2: { x: mid.x + dx, y: mid.y + dy } };
}

export function getTextPosAngle(point1, point2) {
    const mid = getMidPoint(point1, point2);
    let dx = -(point2.y - point1.y) / getDistance(point1, point2);
    let dy = (point2.x - point1.x) / getDistance(point1, point2);
    let angle = getAngle(point1, point2);
    if (point2.x > point1.x)
        return {
            pos: { x: mid.x + dx * 5, y: mid.y + dy * 5 }, angle: - angle
        };
    else
        return {
            pos: { x: mid.x - dx * 5, y: mid.y - dy * 5 }, angle: Math.PI - angle
        };
}

export function snapToGrid(value, gridSpacing) {
    return {
        x: Math.round(value.x / gridSpacing) * gridSpacing,
        y: Math.round(value.y / gridSpacing) * gridSpacing,
    }
}

export function isBoxInside(myArea, otherArea) {
    return (
        otherArea.bl.x >= myArea.bl.x && // otherArea's bottom-left x is to the right of myArea's
        otherArea.bl.y <= myArea.bl.y && // otherArea's bottom-left y is below myArea's
        otherArea.tr.x <= myArea.tr.x && // otherArea's top-right x is to the left of myArea's
        otherArea.tr.y >= myArea.tr.y    // otherArea's top-right y is above myArea's
    );
}

export function add(a, b) {
    return { x: a.x + b.x, y: a.y + b.y }
}
export function sub(a, b) {
    return { x: a.x - b.x, y: a.y - b.y }
}
export function zero() {
    return { x: 0, y: 0 }
}
export function one() {
    return { x: 1, y: 0 }
}
