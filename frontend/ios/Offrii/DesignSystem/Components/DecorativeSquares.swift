import SwiftUI

// MARK: - DecorativeSquares

struct DecorativeSquares: View {
    let preset: Preset

    enum Preset {
        /// 8 elements spread across top portion (h*0.05–0.19), for Login/Register
        case authScreen
        /// 3-4 compact elements adapted for a header (140–200pt)
        case header
        /// 5-6 elements spread across full height, for Onboarding
        case fullScreen
    }

    var body: some View {
        GeometryReader { geometry in
            let width = geometry.size.width
            let height = geometry.size.height

            switch preset {
            case .authScreen:
                authScreenSquares(width: width, height: height)
            case .header:
                headerSquares(width: width, height: height)
            case .fullScreen:
                fullScreenSquares(width: width, height: height)
            }
        }
        .ignoresSafeArea()
    }

    // MARK: - Auth Screen (8 elements)

    @ViewBuilder
    private func authScreenSquares(width: CGFloat, height: CGFloat) -> some View {
        // Left side
        decoSquare(.init("gift.fill", size: 44, opacity: 0.07, iconOpacity: 0.13, corner: 12, rot: -14,
                         grad: (.topLeading, .bottomTrailing), x: 10, y: height * 0.05))
        decoSquare(.init("shippingbox.fill", size: 34, opacity: 0.05, iconOpacity: 0.10, corner: 9, rot: -18,
                         grad: (.topTrailing, .bottomLeading), x: width * 0.22, y: height * 0.10))
        decoSquare(.init("star.fill", size: 38, opacity: 0.06, iconOpacity: 0.11, corner: 10, rot: -8,
                         grad: (.top, .bottom), x: 16, y: height * 0.14))
        decoSquare(.init("balloon.fill", size: 30, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 12,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.15, y: height * 0.19))
        // Right side (mirrored)
        decoSquare(.init("tag.fill", size: 30, opacity: 0.05, iconOpacity: 0.09, corner: 8, rot: 14,
                         grad: (.topTrailing, .bottomLeading), x: width - 40, y: height * 0.05))
        decoSquare(.init("heart.fill", size: 36, opacity: 0.06, iconOpacity: 0.11, corner: 9, rot: 18,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.68, y: height * 0.10))
        decoSquare(.init("sparkle", size: 32, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 8,
                         grad: (.bottomTrailing, .topLeading), x: width - 48, y: height * 0.14))
        decoSquare(.init("wand.and.stars", size: 34, opacity: 0.06, iconOpacity: 0.11, corner: 9, rot: -12,
                         grad: (.topTrailing, .bottomLeading), x: width * 0.76, y: height * 0.19))
    }

    // MARK: - Header (4 compact elements)

    @ViewBuilder
    private func headerSquares(width: CGFloat, height: CGFloat) -> some View {
        decoSquare(.init("gift.fill", size: 30, opacity: 0.06, iconOpacity: 0.11, corner: 8, rot: -12,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.78, y: height * 0.08))
        decoSquare(.init("star.fill", size: 24, opacity: 0.05, iconOpacity: 0.10, corner: 7, rot: 16,
                         grad: (.topTrailing, .bottomLeading), x: width * 0.06, y: height * 0.55))
        decoSquare(.init("sparkle", size: 22, opacity: 0.05, iconOpacity: 0.09, corner: 6, rot: 8,
                         grad: (.bottomTrailing, .topLeading), x: width * 0.62, y: height * 0.68))
        decoSquare(.init("heart.fill", size: 26, opacity: 0.06, iconOpacity: 0.10, corner: 7, rot: -18,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.22, y: height * 0.12))
    }

    // MARK: - Full Screen (6 elements)

    @ViewBuilder
    private func fullScreenSquares(width: CGFloat, height: CGFloat) -> some View {
        decoSquare(.init("gift.fill", size: 40, opacity: 0.06, iconOpacity: 0.11, corner: 11, rot: -14,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.08, y: height * 0.06))
        decoSquare(.init("star.fill", size: 32, opacity: 0.05, iconOpacity: 0.10, corner: 9, rot: 18,
                         grad: (.topTrailing, .bottomLeading), x: width * 0.82, y: height * 0.14))
        decoSquare(.init("heart.fill", size: 36, opacity: 0.06, iconOpacity: 0.11, corner: 10, rot: -8,
                         grad: (.top, .bottom), x: width * 0.12, y: height * 0.38))
        decoSquare(.init("sparkle", size: 28, opacity: 0.05, iconOpacity: 0.09, corner: 8, rot: 12,
                         grad: (.bottomTrailing, .topLeading), x: width * 0.76, y: height * 0.52))
        decoSquare(.init("balloon.fill", size: 34, opacity: 0.06, iconOpacity: 0.10, corner: 9, rot: -16,
                         grad: (.topLeading, .bottomTrailing), x: width * 0.18, y: height * 0.72))
        decoSquare(.init("wand.and.stars", size: 30, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 10,
                         grad: (.topTrailing, .bottomLeading), x: width * 0.70, y: height * 0.85))
    }

    // MARK: - Helper

    private struct SquareConfig {
        let icon: String
        let size: CGFloat
        let opacity: Double
        let iconOpacity: Double
        let corner: CGFloat
        let rot: Double
        let grad: (UnitPoint, UnitPoint)
        let position: CGPoint

        init(_ icon: String, size: CGFloat, opacity: Double, iconOpacity: Double,
             corner: CGFloat, rot: Double, grad: (UnitPoint, UnitPoint), x: CGFloat, y: CGFloat) {
            self.icon = icon
            self.size = size
            self.opacity = opacity
            self.iconOpacity = iconOpacity
            self.corner = corner
            self.rot = rot
            self.grad = grad
            self.position = CGPoint(x: x, y: y)
        }
    }

    private func decoSquare(_ config: SquareConfig) -> some View {
        RoundedRectangle(cornerRadius: config.corner)
            .fill(LinearGradient(colors: [Color.white.opacity(config.opacity), .clear],
                                 startPoint: config.grad.0, endPoint: config.grad.1))
            .frame(width: config.size, height: config.size)
            .overlay(Image(systemName: config.icon)
                .font(.system(size: config.size * 0.4)).foregroundColor(.white.opacity(config.iconOpacity)))
            .rotationEffect(.degrees(config.rot))
            .offset(x: config.position.x, y: config.position.y)
    }
}
