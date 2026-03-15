import SwiftUI
import UIKit

// MARK: - Animation Presets

enum OffriiAnimation {
    /// Standard interactions (buttons, toggles)
    static let defaultSpring: Animation = .spring(response: 0.35, dampingFraction: 0.7)
    /// Tabs, chips, quick selections
    static let snappy: Animation = .spring(response: 0.25, dampingFraction: 0.8)
    /// Shimmer, slow transitions
    static let gentle: Animation = .spring(response: 0.5, dampingFraction: 0.8)
    /// FAB appear, celebrations
    static let bouncy: Animation = .spring(response: 0.4, dampingFraction: 0.6)
    /// Toggle, checkbox
    static let micro: Animation = .spring(response: 0.2, dampingFraction: 0.9)
    /// Pages, modals
    static let modal: Animation = .spring(response: 0.45, dampingFraction: 0.85)
}

// MARK: - Haptic Feedback

@MainActor
enum OffriiHaptics {
    static func tap() {
        UIImpactFeedbackGenerator(style: .light).impactOccurred()
    }

    static func success() {
        UINotificationFeedbackGenerator().notificationOccurred(.success)
    }

    static func error() {
        UINotificationFeedbackGenerator().notificationOccurred(.error)
    }

    static func warning() {
        UINotificationFeedbackGenerator().notificationOccurred(.warning)
    }

    static func selection() {
        UISelectionFeedbackGenerator().selectionChanged()
    }

    static func heavyTap() {
        UIImpactFeedbackGenerator(style: .medium).impactOccurred()
    }
}
