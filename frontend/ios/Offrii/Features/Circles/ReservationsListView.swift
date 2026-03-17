import NukeUI
import SwiftUI

// MARK: - ReservationsListView

struct ReservationsListView: View {
    @State private var reservations: [ReservationResponse] = []
    @State private var isLoading = false

    var body: some View {
        Group {
            if isLoading {
                ScrollView {
                    LazyVStack(spacing: OffriiTheme.spacingSM) {
                        ForEach(0..<4, id: \.self) { _ in
                            SkeletonRow(height: 80)
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingBase)
                    .padding(.top, OffriiTheme.spacingBase)
                }
            } else if reservations.isEmpty {
                Spacer()
                OffriiEmptyState(
                    icon: "gift.fill",
                    title: NSLocalizedString("reservation.empty", comment: ""),
                    subtitle: NSLocalizedString("reservation.emptySubtitle", comment: "")
                )
                Spacer()
            } else {
                ScrollView {
                    LazyVStack(spacing: OffriiTheme.spacingSM) {
                        ForEach(reservations) { reservation in
                            ReservationCard(reservation: reservation)
                        }
                    }
                    .padding(.horizontal, OffriiTheme.spacingBase)
                    .padding(.vertical, OffriiTheme.spacingSM)
                }
            }
        }
        .task { await load() }
    }

    private func load() async {
        isLoading = true
        do {
            reservations = try await CircleService.shared.listReservations()
        } catch {}
        isLoading = false
    }
}

// MARK: - ReservationCard

struct ReservationCard: View {
    let reservation: ReservationResponse

    var body: some View {
        HStack(spacing: OffriiTheme.spacingMD) {
            itemImage
            itemInfo
            Spacer()
            priceAndStatus
        }
        .padding(OffriiTheme.spacingBase)
        .background(OffriiTheme.card)
        .cornerRadius(OffriiTheme.cornerRadiusLG)
        .shadow(color: OffriiTheme.cardShadowColor, radius: 6, x: 0, y: 2)
    }

    @ViewBuilder
    private var itemImage: some View {
        if let urlStr = reservation.itemImageUrl, let url = URL(string: urlStr) {
            LazyImage(url: url) { state in
                if let image = state.image {
                    image
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: 56, height: 56)
                        .cornerRadius(OffriiTheme.cornerRadiusMD)
                        .clipped()
                } else {
                    imagePlaceholder
                }
            }
        } else {
            imagePlaceholder
        }
    }

    private var imagePlaceholder: some View {
        RoundedRectangle(cornerRadius: OffriiTheme.cornerRadiusMD)
            .fill(OffriiTheme.primary.opacity(0.1))
            .frame(width: 56, height: 56)
            .overlay(
                Image(systemName: "gift.fill")
                    .font(.system(size: 20))
                    .foregroundColor(OffriiTheme.primary.opacity(0.4))
            )
    }

    private var itemInfo: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(reservation.itemName)
                .font(OffriiTypography.body)
                .fontWeight(.medium)
                .foregroundColor(OffriiTheme.text)
                .lineLimit(1)

            ownerRow

            if let name = reservation.circleName {
                Text(String(format: NSLocalizedString("reservation.in", comment: ""), name))
                    .font(.system(size: 11))
                    .foregroundColor(OffriiTheme.textMuted)
            }
        }
    }

    private var ownerRow: some View {
        HStack(spacing: 4) {
            if let avatarStr = reservation.ownerAvatarUrl, let url = URL(string: avatarStr) {
                LazyImage(url: url) { state in
                    if let image = state.image {
                        image.resizable().aspectRatio(contentMode: .fill)
                            .frame(width: 16, height: 16).clipShape(Circle())
                    } else {
                        ownerInitial
                    }
                }
            } else {
                ownerInitial
            }

            Text(String(format: NSLocalizedString("reservation.for", comment: ""), reservation.ownerName))
                .font(OffriiTypography.caption)
                .foregroundColor(OffriiTheme.textSecondary)
                .lineLimit(1)
        }
    }

    private var ownerInitial: some View {
        Text(String(reservation.ownerName.prefix(1)).uppercased())
            .font(.system(size: 8, weight: .bold))
            .foregroundColor(.white)
            .frame(width: 16, height: 16)
            .background(OffriiTheme.primary)
            .clipShape(Circle())
    }

    private var priceAndStatus: some View {
        VStack(alignment: .trailing, spacing: 4) {
            if let price = reservation.itemEstimatedPrice {
                Text(formatPrice(price))
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(OffriiTheme.text)
            }

            let isGifted = reservation.itemStatus == "purchased"
            Text(isGifted
                ? NSLocalizedString("reservation.gifted", comment: "")
                : NSLocalizedString("reservation.reserved", comment: ""))
                .font(.system(size: 10, weight: .semibold))
                .foregroundColor(isGifted ? OffriiTheme.success : OffriiTheme.primary)
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background((isGifted ? OffriiTheme.success : OffriiTheme.primary).opacity(0.1))
                .cornerRadius(OffriiTheme.cornerRadiusFull)
        }
    }

    private func formatPrice(_ price: Decimal) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = "EUR"
        return formatter.string(from: price as NSDecimalNumber) ?? "\(price) \u{20AC}"
    }
}
