import NukeUI
import SwiftUI

/// Reusable circle avatar with type badge (person/person.2) overlay.
/// Used in ItemDetailSheet, ItemEditView, and anywhere shared circles are displayed.
struct CircleAvatarBadge: View {
    let name: String
    let isDirect: Bool
    let imageURL: URL?
    var size: CGFloat = 28
    var fontSize: CGFloat = 12

    init(circle: SharedCircleInfo, size: CGFloat = 28, fontSize: CGFloat = 12) {
        self.name = circle.name
        self.isDirect = circle.isDirect ?? false
        self.imageURL = circle.imageURL
        self.size = size
        self.fontSize = fontSize
    }

    init(from circle: OffriiCircle, size: CGFloat = 28, fontSize: CGFloat = 12) {
        self.name = circle.name ?? ""
        self.isDirect = circle.isDirect
        self.imageURL = circle.imageUrl.flatMap { URL(string: $0) }
        self.size = size
        self.fontSize = fontSize
    }

    var body: some View {
        ZStack {
            if let url = imageURL {
                LazyImage(url: url) { state in
                    if let image = state.image {
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                            .frame(width: size, height: size)
                            .clipShape(Circle())
                    } else {
                        initialsView
                    }
                }
            } else {
                initialsView
            }
        }
        .overlay(alignment: .bottomTrailing) {
            Image(systemName: isDirect ? "person.fill" : "person.2.fill")
                .font(.system(size: size * 0.25, weight: .bold))
                .foregroundColor(.white)
                .padding(2)
                .background(OffriiTheme.primary)
                .clipShape(Circle())
                .overlay(Circle().strokeBorder(.white, lineWidth: 1))
                .offset(x: size * 0.1, y: size * 0.1)
        }
    }

    private var initialsView: some View {
        Text(String(name.prefix(1)).uppercased())
            .font(.system(size: fontSize, weight: .bold))
            .foregroundColor(.white)
            .frame(width: size, height: size)
            .background(OffriiTheme.primary)
            .clipShape(Circle())
    }
}
