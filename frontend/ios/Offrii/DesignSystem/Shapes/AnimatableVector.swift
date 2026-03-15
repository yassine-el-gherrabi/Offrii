import SwiftUI

// MARK: - Animatable Control Points

struct AnimatableVector: VectorArithmetic {
    var values: [CGFloat]

    static var zero: AnimatableVector {
        AnimatableVector(values: [])
    }

    static func + (lhs: AnimatableVector, rhs: AnimatableVector) -> AnimatableVector {
        let count = max(lhs.values.count, rhs.values.count)
        var result = [CGFloat](repeating: 0, count: count)
        for idx in 0..<count {
            let lVal = idx < lhs.values.count ? lhs.values[idx] : 0
            let rVal = idx < rhs.values.count ? rhs.values[idx] : 0
            result[idx] = lVal + rVal
        }
        return AnimatableVector(values: result)
    }

    static func - (lhs: AnimatableVector, rhs: AnimatableVector) -> AnimatableVector {
        let count = max(lhs.values.count, rhs.values.count)
        var result = [CGFloat](repeating: 0, count: count)
        for idx in 0..<count {
            let lVal = idx < lhs.values.count ? lhs.values[idx] : 0
            let rVal = idx < rhs.values.count ? rhs.values[idx] : 0
            result[idx] = lVal - rVal
        }
        return AnimatableVector(values: result)
    }

    mutating func scale(by rhs: Double) {
        for i in values.indices {
            values[i] *= CGFloat(rhs)
        }
    }

    var magnitudeSquared: Double {
        values.reduce(0) { $0 + Double($1 * $1) }
    }
}
