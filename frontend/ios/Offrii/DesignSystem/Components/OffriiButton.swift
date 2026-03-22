import SwiftUI

// MARK: - Button Variant

enum OffriiButtonVariant {
    case primary
    case secondary
    case tertiary
    case danger
    case ghost
    case dark
}

// MARK: - OffriiButton

struct OffriiButton: View {
    let title: String
    let variant: OffriiButtonVariant
    let isLoading: Bool
    let isDisabled: Bool
    let action: () -> Void

    @State private var isPressed = false

    init(
        _ title: String,
        variant: OffriiButtonVariant = .primary,
        isLoading: Bool = false,
        isDisabled: Bool = false,
        action: @escaping () -> Void
    ) {
        self.title = title
        self.variant = variant
        self.isLoading = isLoading
        self.isDisabled = isDisabled
        self.action = action
    }

    var body: some View {
        Button(action: {
            guard !isLoading && !isDisabled else { return }
            OffriiHaptics.tap()
            action()
        }) {
            HStack(spacing: OffriiTheme.spacingSM) {
                if isLoading {
                    OffriiSpinner(color: foregroundColor)
                } else {
                    Text(title)
                        .font(OffriiTypography.headline)
                        .foregroundColor(foregroundColor)
                }
            }
            .frame(maxWidth: .infinity)
            .frame(minHeight: 48)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .background(backgroundColor)
            .cornerRadius(OffriiTheme.cornerRadiusMD)
            .overlay(borderOverlay)
        }
        .buttonStyle(.plain)
        .disabled(isDisabled || isLoading)
        .opacity(isDisabled ? 0.5 : 1.0)
        .scaleEffect(isPressed ? 0.97 : 1.0)
        .animation(OffriiAnimation.micro, value: isPressed)
        .animation(OffriiAnimation.defaultSpring, value: isDisabled)
        .animation(OffriiAnimation.defaultSpring, value: isLoading)
        .pressEvents {
            isPressed = true
        } onRelease: {
            isPressed = false
        }
    }

    // MARK: - Computed Style Properties

    private var backgroundColor: Color {
        switch variant {
        case .primary:   return OffriiTheme.primary
        case .secondary: return Color.clear
        case .tertiary:  return OffriiTheme.primaryLight
        case .danger:    return OffriiTheme.danger
        case .ghost:     return Color.clear
        case .dark:      return OffriiTheme.text
        }
    }

    private var foregroundColor: Color {
        switch variant {
        case .primary:   return .white
        case .secondary: return OffriiTheme.primary
        case .tertiary:  return OffriiTheme.primary
        case .danger:    return .white
        case .ghost:     return OffriiTheme.primary
        case .dark:      return .white
        }
    }

    @ViewBuilder
    private var borderOverlay: some View {
        switch variant {
        case .secondary:
            RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
                .strokeBorder(OffriiTheme.primary, lineWidth: 1.5)
        default:
            EmptyView()
        }
    }
}
