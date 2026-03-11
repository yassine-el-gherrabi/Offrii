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
        for i in 0..<count {
            let l = i < lhs.values.count ? lhs.values[i] : 0
            let r = i < rhs.values.count ? rhs.values[i] : 0
            result[i] = l + r
        }
        return AnimatableVector(values: result)
    }

    static func - (lhs: AnimatableVector, rhs: AnimatableVector) -> AnimatableVector {
        let count = max(lhs.values.count, rhs.values.count)
        var result = [CGFloat](repeating: 0, count: count)
        for i in 0..<count {
            let l = i < lhs.values.count ? lhs.values[i] : 0
            let r = i < rhs.values.count ? rhs.values[i] : 0
            result[i] = l - r
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
