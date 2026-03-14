import SwiftUI

// MARK: - Header Variant

enum HeaderVariant {
    case home
    case envies
    case cercles
    case entraide
    case profil
    case detail

    var gradient: [Color] {
        switch self {
        case .home:     return [OffriiTheme.primary, OffriiTheme.primaryLight]
        case .envies:   return [OffriiTheme.primary, OffriiTheme.primary.opacity(0.85)]
        case .cercles:  return [OffriiTheme.secondary, OffriiTheme.secondary.opacity(0.85)]
        case .entraide: return [OffriiTheme.accent, OffriiTheme.primary.opacity(0.8)]
        case .profil:   return [OffriiTheme.primary, OffriiTheme.primary.opacity(0.9)]
        case .detail:   return [OffriiTheme.accent.opacity(0.8), OffriiTheme.accent.opacity(0.6)]
        }
    }

    var blobPreset: BlobPreset {
        switch self {
        case .home:     return .home
        case .envies:   return .envies
        case .cercles:  return .cercles
        case .entraide: return .entraide
        case .profil:   return .profil
        case .detail:   return .envies
        }
    }

    var height: CGFloat {
        switch self {
        case .home:   return 120
        case .detail: return 100
        default:      return 160
        }
    }
}

// MARK: - SectionHeader

struct SectionHeader<TrailingContent: View>: View {
    let title: String
    var subtitle: String?
    let variant: HeaderVariant
    let trailing: TrailingContent

    init(
        title: String,
        subtitle: String? = nil,
        variant: HeaderVariant,
        @ViewBuilder trailing: () -> TrailingContent = { EmptyView() }
    ) {
        self.title = title
        self.subtitle = subtitle
        self.variant = variant
        self.trailing = trailing()
    }

    var body: some View {
        ZStack {
            // Gradient background
            LinearGradient(
                colors: variant.gradient,
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea(edges: .top)

            // Blob decorations
            BlobBackground(preset: variant.blobPreset)
                .opacity(0.5)

            // Content
            HStack(alignment: .bottom) {
                VStack(alignment: .leading, spacing: OffriiTheme.spacingXS) {
                    Text(title)
                        .font(variant == .detail ? OffriiTypography.titleMedium : OffriiTypography.displayLarge)
                        .foregroundColor(.white)

                    if let subtitle {
                        Text(subtitle)
                            .font(OffriiTypography.subheadline)
                            .foregroundColor(.white.opacity(0.8))
                    }
                }

                Spacer()

                trailing
            }
            .padding(.horizontal, OffriiTheme.spacingLG)
            .padding(.bottom, OffriiTheme.spacingLG)
            .padding(.top, variant == .detail ? OffriiTheme.spacingMD : OffriiTheme.spacingXL)
        }
        .frame(minHeight: variant.height)
    }
}
