import SwiftUI
import UIKit

/// Wraps UIKit's UIVisualEffectView for real GPU-accelerated blur.
/// Usage: `BlurView(style: .systemUltraThinMaterialDark)`
struct BlurView: UIViewRepresentable {
    var style: UIBlurEffect.Style

    func makeUIView(context: Context) -> UIVisualEffectView {
        UIVisualEffectView(effect: UIBlurEffect(style: style))
    }

    func updateUIView(_ uiView: UIVisualEffectView, context: Context) {
        uiView.effect = UIBlurEffect(style: style)
    }
}
