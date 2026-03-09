import SwiftUI

// MARK: - Button Variant

enum OffriiButtonVariant {
    case primary
    case secondary
    case danger
    case dark
}

// MARK: - OffriiButton

struct OffriiButton: View {
    let title: String
    let variant: OffriiButtonVariant
    let isLoading: Bool
    let isDisabled: Bool
    let action: () -> Void

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
            action()
        }) {
            HStack(spacing: OffriiTheme.spacingSM) {
                if isLoading {
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle(tint: foregroundColor))
                }

                Text(title)
                    .font(OffriiTypography.headline)
                    .foregroundColor(foregroundColor)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, OffriiTheme.spacingMD)
            .padding(.horizontal, OffriiTheme.spacingLG)
            .background(backgroundColor)
            .cornerRadius(buttonCornerRadius)
            .overlay(borderOverlay)
        }
        .disabled(isDisabled || isLoading)
        .opacity(isDisabled ? 0.5 : 1.0)
        .animation(OffriiTheme.defaultAnimation, value: isDisabled)
        .animation(OffriiTheme.defaultAnimation, value: isLoading)
    }

    // MARK: - Computed Style Properties

    private var backgroundColor: Color {
        switch variant {
        case .primary:
            return OffriiTheme.primary
        case .secondary:
            return Color.clear
        case .danger:
            return OffriiTheme.danger
        case .dark:
            return OffriiTheme.text
        }
    }

    private var foregroundColor: Color {
        switch variant {
        case .primary:
            return .white
        case .secondary:
            return OffriiTheme.primary
        case .danger:
            return .white
        case .dark:
            return .white
        }
    }

    private var buttonCornerRadius: CGFloat {
        switch variant {
        case .dark:
            return OffriiTheme.cornerRadiusXL
        default:
            return OffriiTheme.cornerRadiusMD
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

// MARK: - Preview

#if DEBUG
struct OffriiButton_Previews: PreviewProvider {
    static var previews: some View {
        VStack(spacing: OffriiTheme.spacingMD) {
            OffriiButton("Valider", variant: .primary) {}
            OffriiButton("Annuler", variant: .secondary) {}
            OffriiButton("Supprimer", variant: .danger) {}
            OffriiButton("Chargement...", variant: .primary, isLoading: true) {}
            OffriiButton("Indisponible", variant: .primary, isDisabled: true) {}
        }
        .padding(OffriiTheme.spacingLG)
        .background(OffriiTheme.cardSurface)
        .previewLayout(.sizeThatFits)
    }
}
#endif
