import SwiftUI

// MARK: - OffriiFloatingActionButton

struct OffriiFloatingActionButton: View {
    let icon: String
    var label: String?
    let action: () -> Void
    @State private var isPressed = false
    @State private var appeared = false

    var body: some View {
        Button(action: {
            OffriiHaptics.heavyTap()
            action()
        }) {
            HStack(spacing: OffriiTheme.spacingSM) {
                Image(systemName: icon)
                    .font(.system(size: 20, weight: .semibold))

                if let label {
                    Text(label)
                        .font(OffriiTypography.headline)
                }
            }
            .foregroundColor(.white)
            .padding(.horizontal, label != nil ? OffriiTheme.spacingLG : 0)
            .frame(width: label != nil ? nil : 56, height: 56)
            .background(OffriiTheme.primary)
            .cornerRadius(label != nil ? OffriiTheme.cornerRadiusFull : 28)
            .shadow(
                color: OffriiTheme.primary.opacity(0.35),
                radius: 16, x: 0, y: 6
            )
        }
        .buttonStyle(.plain)
        .scaleEffect(isPressed ? 0.92 : (appeared ? 1.0 : 0.5))
        .opacity(appeared ? 1 : 0)
        .onAppear {
            withAnimation(OffriiAnimation.bouncy) {
                appeared = true
            }
        }
        .pressEvents {
            withAnimation(OffriiAnimation.micro) { isPressed = true }
        } onRelease: {
            withAnimation(OffriiAnimation.micro) { isPressed = false }
        }
    }
}

// MARK: - Press Events Modifier

struct PressEventsModifier: ViewModifier {
    var onPress: () -> Void
    var onRelease: () -> Void

    func body(content: Content) -> some View {
        content
            .simultaneousGesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { _ in onPress() }
                    .onEnded { _ in onRelease() }
            )
    }
}

extension View {
    func pressEvents(onPress: @escaping () -> Void, onRelease: @escaping () -> Void) -> some View {
        modifier(PressEventsModifier(onPress: onPress, onRelease: onRelease))
    }
}
