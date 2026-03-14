import SwiftUI

// MARK: - Blob Shape

struct BlobShape: Shape {
    var controlPoints: AnimatableVector

    var animatableData: AnimatableVector {
        get { controlPoints }
        set { controlPoints = newValue }
    }

    func path(in rect: CGRect) -> Path {
        let w = rect.width
        let h = rect.height

        // Use 8 control point offsets (4 pairs of x,y deltas)
        let cp = controlPoints.values
        let d0x = !cp.isEmpty ? cp[0] : 0
        let d0y = cp.count > 1 ? cp[1] : 0
        let d1x = cp.count > 2 ? cp[2] : 0
        let d1y = cp.count > 3 ? cp[3] : 0
        let d2x = cp.count > 4 ? cp[4] : 0
        let d2y = cp.count > 5 ? cp[5] : 0
        let d3x = cp.count > 6 ? cp[6] : 0
        let d3y = cp.count > 7 ? cp[7] : 0

        var path = Path()

        // Organic blob using cubic bezier curves
        path.move(to: CGPoint(x: w * 0.5, y: h * 0.0))

        path.addCurve(
            to: CGPoint(x: w * 1.0, y: h * 0.4),
            control1: CGPoint(x: w * 0.75 + d0x, y: h * 0.0 + d0y),
            control2: CGPoint(x: w * 1.0 + d1x, y: h * 0.15 + d1y)
        )

        path.addCurve(
            to: CGPoint(x: w * 0.6, y: h * 1.0),
            control1: CGPoint(x: w * 1.0 + d2x, y: h * 0.7 + d2y),
            control2: CGPoint(x: w * 0.85 + d3x, y: h * 1.0 + d3y)
        )

        path.addCurve(
            to: CGPoint(x: w * 0.0, y: h * 0.6),
            control1: CGPoint(x: w * 0.3 - d3x, y: h * 1.0 - d3y),
            control2: CGPoint(x: w * 0.0 - d2x, y: h * 0.85 - d2y)
        )

        path.addCurve(
            to: CGPoint(x: w * 0.5, y: h * 0.0),
            control1: CGPoint(x: w * 0.0 - d1x, y: h * 0.3 - d1y),
            control2: CGPoint(x: w * 0.25 - d0x, y: h * 0.0 - d0y)
        )

        path.closeSubpath()
        return path
    }

    // MARK: - Presets

    static var idle: AnimatableVector {
        AnimatableVector(values: [0, 0, 0, 0, 0, 0, 0, 0])
    }

    static var morphed: AnimatableVector {
        AnimatableVector(values: [8, -6, -5, 7, 6, -8, -7, 5])
    }
}

// MARK: - Animated Blob View

struct AnimatedBlobView: View {
    let color: Color
    let size: CGFloat
    @State private var isMorphed = false

    var body: some View {
        BlobShape(controlPoints: isMorphed ? BlobShape.morphed : BlobShape.idle)
            .fill(color)
            .frame(width: size, height: size)
            .onAppear {
                withAnimation(
                    .easeInOut(duration: 8)
                    .repeatForever(autoreverses: true)
                ) {
                    isMorphed = true
                }
            }
    }
}

// MARK: - Section Blob Backgrounds

struct BlobConfig {
    let color: Color
    let size: CGFloat
    let opacity: Double
    let offsetX: CGFloat
    let offsetY: CGFloat
}

enum BlobPreset {
    case auth
    case home
    case envies
    case cercles
    case entraide
    case profil

    var blobs: [BlobConfig] {
        switch self {
        case .auth:
            return [
                BlobConfig(color: OffriiTheme.primary, size: 280, opacity: 0.08, offsetX: 120, offsetY: -80),
                BlobConfig(color: OffriiTheme.accent, size: 220, opacity: 0.06, offsetX: -100, offsetY: 200),
                BlobConfig(color: Color(red: 0.96, green: 0.68, blue: 0.22), size: 200, opacity: 0.07, offsetX: -110, offsetY: -200),
            ]
        case .home:
            return [
                BlobConfig(color: OffriiTheme.primary, size: 200, opacity: 0.08, offsetX: 120, offsetY: -30),
                BlobConfig(color: OffriiTheme.primaryLight, size: 160, opacity: 0.06, offsetX: -80, offsetY: 20),
            ]
        case .envies:
            return [
                BlobConfig(color: OffriiTheme.primary, size: 240, opacity: 0.10, offsetX: 140, offsetY: -40),
            ]
        case .cercles:
            return [
                BlobConfig(color: OffriiTheme.secondary, size: 180, opacity: 0.08, offsetX: 120, offsetY: -30),
                BlobConfig(color: OffriiTheme.accent, size: 150, opacity: 0.06, offsetX: -60, offsetY: 40),
            ]
        case .entraide:
            return [
                BlobConfig(color: OffriiTheme.accent, size: 300, opacity: 0.08, offsetX: 0, offsetY: -20),
                BlobConfig(color: OffriiTheme.primary, size: 200, opacity: 0.06, offsetX: 100, offsetY: 60),
            ]
        case .profil:
            return [
                BlobConfig(color: OffriiTheme.primary, size: 200, opacity: 0.07, offsetX: 0, offsetY: 0),
            ]
        }
    }
}

struct BlobBackground: View {
    let preset: BlobPreset

    var body: some View {
        ZStack {
            ForEach(Array(preset.blobs.enumerated()), id: \.offset) { _, blob in
                AnimatedBlobView(
                    color: blob.color.opacity(blob.opacity),
                    size: blob.size
                )
                .offset(x: blob.offsetX, y: blob.offsetY)
            }
        }
    }
}
