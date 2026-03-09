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
            let w = geometry.size.width
            let h = geometry.size.height

            switch preset {
            case .authScreen:
                authScreenSquares(w: w, h: h)
            case .header:
                headerSquares(w: w, h: h)
            case .fullScreen:
                fullScreenSquares(w: w, h: h)
            }
        }
        .ignoresSafeArea()
    }

    // MARK: - Auth Screen (8 elements)

    @ViewBuilder
    private func authScreenSquares(w: CGFloat, h: CGFloat) -> some View {
        // Left side
        decoSquare("gift.fill", size: 44, opacity: 0.07, iconOpacity: 0.13, corner: 12, rot: -14,
                   grad: (.topLeading, .bottomTrailing), x: 10, y: h * 0.05)
        decoSquare("shippingbox.fill", size: 34, opacity: 0.05, iconOpacity: 0.10, corner: 9, rot: -18,
                   grad: (.topTrailing, .bottomLeading), x: w * 0.22, y: h * 0.10)
        decoSquare("star.fill", size: 38, opacity: 0.06, iconOpacity: 0.11, corner: 10, rot: -8,
                   grad: (.top, .bottom), x: 16, y: h * 0.14)
        decoSquare("balloon.fill", size: 30, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 12,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.15, y: h * 0.19)
        // Right side (mirrored)
        decoSquare("tag.fill", size: 30, opacity: 0.05, iconOpacity: 0.09, corner: 8, rot: 14,
                   grad: (.topTrailing, .bottomLeading), x: w - 40, y: h * 0.05)
        decoSquare("heart.fill", size: 36, opacity: 0.06, iconOpacity: 0.11, corner: 9, rot: 18,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.68, y: h * 0.10)
        decoSquare("sparkle", size: 32, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 8,
                   grad: (.bottomTrailing, .topLeading), x: w - 48, y: h * 0.14)
        decoSquare("wand.and.stars", size: 34, opacity: 0.06, iconOpacity: 0.11, corner: 9, rot: -12,
                   grad: (.topTrailing, .bottomLeading), x: w * 0.76, y: h * 0.19)
    }

    // MARK: - Header (4 compact elements)

    @ViewBuilder
    private func headerSquares(w: CGFloat, h: CGFloat) -> some View {
        decoSquare("gift.fill", size: 30, opacity: 0.06, iconOpacity: 0.11, corner: 8, rot: -12,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.78, y: h * 0.08)
        decoSquare("star.fill", size: 24, opacity: 0.05, iconOpacity: 0.10, corner: 7, rot: 16,
                   grad: (.topTrailing, .bottomLeading), x: w * 0.06, y: h * 0.55)
        decoSquare("sparkle", size: 22, opacity: 0.05, iconOpacity: 0.09, corner: 6, rot: 8,
                   grad: (.bottomTrailing, .topLeading), x: w * 0.62, y: h * 0.68)
        decoSquare("heart.fill", size: 26, opacity: 0.06, iconOpacity: 0.10, corner: 7, rot: -18,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.22, y: h * 0.12)
    }

    // MARK: - Full Screen (6 elements)

    @ViewBuilder
    private func fullScreenSquares(w: CGFloat, h: CGFloat) -> some View {
        decoSquare("gift.fill", size: 40, opacity: 0.06, iconOpacity: 0.11, corner: 11, rot: -14,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.08, y: h * 0.06)
        decoSquare("star.fill", size: 32, opacity: 0.05, iconOpacity: 0.10, corner: 9, rot: 18,
                   grad: (.topTrailing, .bottomLeading), x: w * 0.82, y: h * 0.14)
        decoSquare("heart.fill", size: 36, opacity: 0.06, iconOpacity: 0.11, corner: 10, rot: -8,
                   grad: (.top, .bottom), x: w * 0.12, y: h * 0.38)
        decoSquare("sparkle", size: 28, opacity: 0.05, iconOpacity: 0.09, corner: 8, rot: 12,
                   grad: (.bottomTrailing, .topLeading), x: w * 0.76, y: h * 0.52)
        decoSquare("balloon.fill", size: 34, opacity: 0.06, iconOpacity: 0.10, corner: 9, rot: -16,
                   grad: (.topLeading, .bottomTrailing), x: w * 0.18, y: h * 0.72)
        decoSquare("wand.and.stars", size: 30, opacity: 0.05, iconOpacity: 0.10, corner: 8, rot: 10,
                   grad: (.topTrailing, .bottomLeading), x: w * 0.70, y: h * 0.85)
    }

    // MARK: - Helper

    private func decoSquare(
        _ icon: String, size: CGFloat, opacity: Double, iconOpacity: Double,
        corner: CGFloat, rot: Double,
        grad: (UnitPoint, UnitPoint), x: CGFloat, y: CGFloat
    ) -> some View {
        RoundedRectangle(cornerRadius: corner)
            .fill(LinearGradient(colors: [Color.white.opacity(opacity), .clear],
                                 startPoint: grad.0, endPoint: grad.1))
            .frame(width: size, height: size)
            .overlay(Image(systemName: icon)
                .font(.system(size: size * 0.4)).foregroundColor(.white.opacity(iconOpacity)))
            .rotationEffect(.degrees(rot))
            .offset(x: x, y: y)
    }
}
